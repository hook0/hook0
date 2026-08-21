"""What the generated request layer puts on the wire, and what it does with what comes back.

The generated half is handed a transport and nothing else, so these cases hand it the real ones and
watch a real API answer: the path it interpolated, the query it assembled, the credential it
carried, the type it read back, and the exception it raised when the answer was a problem.

Nothing here names an operation, a group or a schema. The groups are found on the generated modules,
their methods and the arguments each one takes are read off the methods themselves, and what every
call is held to is read off the API document the generator was run against. An operation the API
grows is therefore driven the moment the generated modules carry it, and one it loses takes its
case with it — including the case that says every operation the document declares was reached.
"""

from __future__ import annotations

import asyncio
import dataclasses
import datetime
import inspect
import itertools
import json
import types
import typing
import urllib.parse
import uuid
from enum import StrEnum
from pathlib import Path
from typing import Any

import pytest
from conftest import FakeHook0Api, ReceivedRequest, ScriptedResponse, TEST_TIMEOUT

from hook0 import AsyncHttpTransport, HttpTransport
from hook0.generated import aio as awaiting_flavour
from hook0.generated import api as blocking_flavour
from hook0.generated import models
from hook0.generated.errors import NotFoundError, PROBLEMS, ProblemError
from hook0.runtime import DecodeError

pytestmark = pytest.mark.timeout(TEST_TIMEOUT)

# No repository nests this package deeper than this below its root; the bound turns a checkout that
# is missing the document into a failure rather than a walk up to `/`.
MAX_ANCESTORS = 8

# No schema the API declares nests anywhere near this deep. The bound turns a document that
# describes itself into a failure instead of a recursion that never returns.
MAX_DEPTH = 8

# The methods a request line can carry, which is what tells an operation apart from the rest of what
# a path item holds.
VERBS = frozenset({"get", "put", "post", "delete", "options", "head", "patch", "trace"})

# The tag that marks an operation as part of the surface an SDK exposes. A document that marks none
# of its operations with it declares the whole of itself part of that surface, which is the rule the generator
# applies and therefore the rule this suite holds it to.
SDK_TAG = "sdk"

# What every string-shaped argument is given. It carries the two characters a path segment may not
# leave as they are, so that a value reaching a path proves it was escaped rather than pasted.
A_STRING = "a value/with a space"

# What a value the document does not describe is answered as: kept as it arrived, whatever it is.
AN_OPAQUE_VALUE = {"the document": ["describes", "none of this"]}


def _api_document() -> Any:
    """The OpenAPI document the generator was run against, out of the repository holding it."""
    for parent in itertools.islice(Path(__file__).resolve().parents, MAX_ANCESTORS):
        candidate = parent / "api" / "openapi.snapshot.json"
        if candidate.is_file():
            return json.loads(candidate.read_text(encoding="utf-8"))
    raise AssertionError(f"no `api/openapi.snapshot.json` within {MAX_ANCESTORS} directories of {__file__}")


@dataclasses.dataclass(frozen=True)
class Operation:
    """One operation the API document declares, as a request has to look to be it."""

    method: str
    # The path with its parameters still written as the document writes them, `{like_this}`.
    template: str
    required_query: frozenset[str]
    optional_query: frozenset[str]

    def matches(self, target: str) -> bool:
        """Whether a request line landed on this operation."""
        wanted = self.template.split("/")
        got = urllib.parse.urlsplit(target).path.split("/")
        if len(wanted) != len(got):
            return False
        for declared, sent in zip(wanted, got, strict=True):
            if declared.startswith("{") and declared.endswith("}"):
                # A parameter stands for a segment that is there; an empty one is the trailing
                # slash of another path rather than a value.
                if not sent:
                    return False
                continue
            if declared != sent:
                return False
        return True

    def parameters(self) -> frozenset[str]:
        return self.required_query | self.optional_query


def _declared_operations() -> list[Operation]:
    """Every operation an SDK is built out of, which is what the document marks with the SDK tag."""
    document = _api_document()
    found = []
    for template, item in document["paths"].items():
        for verb, operation in item.items():
            if verb not in VERBS:
                continue
            declared = operation.get("parameters", [])
            query = [parameter for parameter in declared if parameter.get("in") == "query"]
            found.append(
                (
                    SDK_TAG in operation.get("tags", []),
                    Operation(
                        method=verb.upper(),
                        template=template,
                        required_query=frozenset(p["name"] for p in query if p.get("required")),
                        optional_query=frozenset(p["name"] for p in query if not p.get("required")),
                    ),
                )
            )
    if not found:
        raise AssertionError("the API document declares no operation at all")

    # A document that marks nothing public exposes all of itself; one that marks anything exposes
    # what it marked. Both are what the generator does with the tag, so both are what a target is
    # held to here.
    if any(public for public, _ in found):
        return [operation for public, operation in found if public]
    return [operation for _, operation in found]


OPERATIONS = _declared_operations()


def _peel(annotation: Any) -> tuple[bool, Any]:
    """Whether a value may be absent, and what it is when it is not."""
    if typing.get_origin(annotation) in (types.UnionType, typing.Union):
        carried = [arm for arm in typing.get_args(annotation) if arm is not type(None)]
        if len(carried) != len(typing.get_args(annotation)):
            return True, carried[0]
    return False, annotation


def _python_value(annotation: Any, *, optionals: bool, depth: int) -> Any:
    """One value of the type a field or an argument declares, as Python holds it."""
    if depth > MAX_DEPTH:
        raise AssertionError(f"{annotation} nests more than {MAX_DEPTH} deep")

    origin = typing.get_origin(annotation)
    if origin is list:
        (item,) = typing.get_args(annotation)
        return [_python_value(item, optionals=optionals, depth=depth + 1)]
    if origin is dict:
        _, item = typing.get_args(annotation)
        return {"a key": _python_value(item, optionals=optionals, depth=depth + 1)}

    if annotation is Any:
        return AN_OPAQUE_VALUE
    if annotation is str:
        return A_STRING
    if annotation is bool:
        return True
    if annotation is int:
        return 12
    if annotation is float:
        return 1.5
    if annotation is uuid.UUID:
        return uuid.UUID("3f2504e0-4f89-41d3-9a0c-0305e82c3301")
    if annotation is datetime.datetime:
        return datetime.datetime(2026, 1, 2, 3, 4, 5, tzinfo=datetime.UTC)
    if annotation is datetime.date:
        return datetime.date(2026, 1, 2)
    if isinstance(annotation, type) and issubclass(annotation, StrEnum):
        # The first value the list declares: which one is immaterial, that it is one of them is not.
        return next(iter(annotation))
    if dataclasses.is_dataclass(annotation):
        return _model(annotation, optionals=optionals, depth=depth + 1)

    raise AssertionError(f"the generated modules carry a {annotation} nothing here knows how to build")


def _model(kind: Any, *, optionals: bool, depth: int = 0) -> Any:
    """One value of a schema the API declares, with every member it may leave out either set or not."""
    hints = typing.get_type_hints(kind)
    held = {}
    for field in dataclasses.fields(kind):
        absent, carried = _peel(hints[field.name])
        held[field.name] = (
            None if absent and not optionals else _python_value(carried, optionals=optionals, depth=depth)
        )
    return kind(**held)


def _models() -> list[Any]:
    """Every schema the generator wrote a class for."""
    found = [
        declared
        for declared in vars(models).values()
        if dataclasses.is_dataclass(declared) and declared.__module__ == models.__name__
    ]
    if not found:
        raise AssertionError("the generator wrote no schema at all")
    return found


def _enumerations() -> list[Any]:
    """Every closed list of strings the generator wrote."""
    found = [
        declared
        for declared in vars(models).values()
        if isinstance(declared, type) and issubclass(declared, StrEnum) and declared.__module__ == models.__name__
    ]
    if not found:
        raise AssertionError("the generator wrote no closed list of strings at all")
    return found


def _groups(module: Any) -> list[Any]:
    """Every group of operations the generator wrote in one module."""
    found = [
        declared
        for declared in vars(module).values()
        if inspect.isclass(declared) and declared.__module__ == module.__name__
    ]
    if not found:
        raise AssertionError(f"{module.__name__} carries no group of operations")
    return found


def _methods(group: Any) -> list[tuple[str, Any]]:
    """Every operation one group carries, under the name it is called by."""
    return sorted(
        (name, declared) for name, declared in vars(group).items() if callable(declared) and not name.startswith("_")
    )


def _answer_for(method: Any, *, optionals: bool) -> tuple[Any, Any]:
    """What the API answers this method, and the value the method is expected to read out of it."""
    returned = typing.get_type_hints(method).get("return")
    if returned is None or returned is type(None):
        return {}, None
    if typing.get_origin(returned) is list:
        (item,) = typing.get_args(returned)
        if item is str:
            return ["a value the API listed"], ["a value the API listed"]
        held = _model(item, optionals=optionals)
        return [held.to_json()], [held]
    held = _model(returned, optionals=optionals)
    return held.to_json(), held


def _arguments(method: Any, *, optionals: bool) -> dict[str, Any]:
    """What one operation is asked with: everything it requires, and what it does not as asked for."""
    hints = typing.get_type_hints(method)
    given: dict[str, Any] = {}
    for name in itertools.islice(inspect.signature(method).parameters, 1, None):
        absent, carried = _peel(hints[name])
        if absent and not optionals:
            continue
        given[name] = _python_value(carried, optionals=optionals, depth=0)
    return given


def _called(group: Any, method: Any, given: dict[str, Any]) -> Any:
    """One operation, asked the same thing whichever flavour of the generated modules carries it."""
    answered = method(group, **given)
    if inspect.isawaitable(answered):
        return asyncio.run(answered)
    return answered


def _operation_for(request: ReceivedRequest) -> Operation:
    """Which operation of the document a request landed on."""
    matched = [
        operation
        for operation in OPERATIONS
        if operation.method == request.method and operation.matches(request.target)
    ]
    if len(matched) != 1:
        raise AssertionError(f"`{request.method} {request.target}` is {len(matched)} of the operations declared")
    return matched[0]


def _query_of(request: ReceivedRequest) -> dict[str, list[str]]:
    return urllib.parse.parse_qs(urllib.parse.urlsplit(request.target).query, keep_blank_values=True)


def _built(module: Any, group: Any, base_url: str) -> Any:
    transport = AsyncHttpTransport if module is awaiting_flavour else HttpTransport
    return group(transport(base_url, "token-xyz"))


def _applications(module: Any, base_url: str) -> tuple[Any, Any]:
    """The one group every case below asks the same thing of, and the method it asks for.

    Which group it is does not matter — what these cases are about is what happens to an answer,
    not which operation drew it — so it is found rather than named: the first group carrying a
    method that reads back a value of its own.
    """
    for group in _groups(module):
        for _, method in _methods(group):
            returned = typing.get_type_hints(method).get("return")
            asked = _arguments(method, optionals=False)
            if returned not in (None, type(None)) and typing.get_origin(returned) is not list and asked:
                return _built(module, group, base_url), method
    raise AssertionError(f"{module.__name__} carries no operation that reads back a value of its own")


@pytest.fixture(params=[blocking_flavour, awaiting_flavour], ids=["blocking", "awaiting"])
def generated(request: Any) -> Any:
    """Which flavour of the generated modules a case is running against."""
    return request.param


@pytest.mark.parametrize("optionals", [True, False], ids=["with-optional-arguments", "without-them"])
def test_every_operation_the_document_declares_is_reached_the_way_it_declares_it(
    api: FakeHook0Api, generated: Any, optionals: bool
) -> None:
    """Every generated method issues the request its operation is declared as, and reads its answer back.

    Run twice: once giving every argument the operation may be asked with, once giving only the ones
    it requires, which is what says an argument left out leaves the query it would have filled empty.
    """
    reached: set[Operation] = set()

    for group in _groups(generated):
        reaching = _built(generated, group, api.base_url)
        for name, method in _methods(group):
            given = _arguments(method, optionals=optionals)
            answered, expected = _answer_for(method, optionals=optionals)
            api.will_answer(ScriptedResponse(200, answered))

            read = _called(reaching, method, given)

            request = api.received[-1]
            operation = _operation_for(request)
            named = f"{group.__name__}.{name}"

            assert read == expected, f"{named} did not read back what the API answered"
            assert request.headers["authorization"] == "Bearer token-xyz", f"{named} carried no credential"
            assert request.headers["accept"] == "application/json", f"{named} asked for no representation"

            sent = urllib.parse.urlsplit(request.target).path.split("/")
            for declared, segment in zip(operation.template.split("/"), sent, strict=True):
                if declared.startswith("{"):
                    # The value lands in the path escaped, so that nothing in it can name a segment
                    # the operation never had.
                    assert segment == urllib.parse.quote(A_STRING, safe=""), f"{named} left `{declared}` unescaped"

            wanted = operation.parameters() if optionals else operation.required_query
            carried = _query_of(request)
            assert set(carried) == wanted, f"{named} assembled a query the document does not declare"
            for parameter in wanted:
                assert carried[parameter] == [A_STRING], f"{named} carried `{parameter}` altered"

            if "body" in given:
                assert request.json() == given["body"].to_json(), f"{named} sent a body the API cannot read back"

            reached.add(operation)

    assert reached == set(OPERATIONS), "the generated modules reach fewer operations than the API declares"


@pytest.mark.parametrize("optionals", [True, False], ids=["with-optional-members", "without-them"])
def test_every_schema_the_document_declares_reads_back_what_it_wrote(optionals: bool) -> None:
    """A value written out and read back in is the value it started as, member for member.

    Run once with every member the schema may leave out set and once with none of them, which is
    what tells a member that was read apart from one that was defaulted to the same thing.
    """
    for kind in _models():
        held = _model(kind, optionals=optionals)
        written = held.to_json()

        assert isinstance(written, dict), f"{kind.__name__} does not write itself as a JSON object"
        assert kind.from_json(written) == held, f"{kind.__name__} does not read back what it wrote"
        assert kind.from_json(written).to_json() == written, f"{kind.__name__} does not write back what it read"

        unset = {field.name for field in dataclasses.fields(kind) if getattr(held, field.name) is None}
        if optionals:
            assert not unset, f"{kind.__name__} left a member unset that the case set"
        for name in unset:
            assert name not in written, f"{kind.__name__} wrote `{name}` out although it holds nothing"


def test_a_schema_refuses_a_document_that_is_not_the_object_it_declares() -> None:
    """Whatever the API answers, a schema either reads it or says it could not."""
    for kind in _models():
        for answered in (1, "text", True, [], None):
            with pytest.raises(DecodeError):
                kind.from_json(answered)


def test_a_schema_refuses_a_member_the_document_does_not_declare_it_as() -> None:
    """A member answered as something else stops the read, and says which member it was.

    Everything a schema describes is refused when it arrives as something else. What it leaves
    undescribed is kept as it arrived and so is refused by nothing, and there are exactly as many
    members that accept anything as the schema declares undescribed.
    """
    for kind in _models():
        hints = typing.get_type_hints(kind)
        opaque = sum(1 for field in dataclasses.fields(kind) if _peel(hints[field.name])[1] is Any)

        written = _model(kind, optionals=True).to_json()
        accepted = []
        for name in written:
            # Neither an object nor a scalar any of the readers accept, whichever the member is.
            wrong = dict(written) | {name: [{"neither": "a scalar"}]}
            try:
                kind.from_json(wrong)
            except DecodeError as refused:
                assert name in str(refused), f"{kind.__name__} did not say which member it could not read"
            else:
                accepted.append(name)

        assert len(accepted) == opaque, f"{kind.__name__} read {accepted} although it describes what they hold"


def test_every_closed_list_reads_the_values_it_declares_and_no_other() -> None:
    for kind in _enumerations():
        declared = list(kind)
        assert declared, f"{kind.__name__} declares no value at all"
        for value in declared:
            assert kind(value.value) is value, f"{kind.__name__} does not read `{value.value}` back"
        with pytest.raises(ValueError):
            kind("a value the API never declared")


def test_every_problem_the_document_names_is_raised_as_its_own_failure(api: FakeHook0Api, generated: Any) -> None:
    """A caller may catch the one problem it cares about, or every problem, and both work."""
    reaching, method = _applications(generated, api.base_url)
    asked = _arguments(method, optionals=False)

    for problem, expected in PROBLEMS.items():
        api.will_answer(
            ScriptedResponse(
                400,
                {
                    "id": problem.value,
                    "status": 400,
                    "title": "refused",
                    "detail": "what the case scripted",
                    "type": f"https://hook0.com/documentation/errors/{problem.value}",
                },
            )
        )

        with pytest.raises(expected) as reported:
            _called(reaching, method, asked)

        assert reported.value.status == 400
        assert reported.value.problem is not None
        assert reported.value.problem.id_ is problem
        assert reported.value.problem.detail == "what the case scripted"
        assert isinstance(reported.value, ProblemError)


def test_a_problem_this_client_has_never_heard_of_is_still_reported_as_a_failure(
    api: FakeHook0Api, generated: Any
) -> None:
    reaching, method = _applications(generated, api.base_url)
    api.will_answer(ScriptedResponse(400, {"id": "AProblemThisClientHasNeverHeardOf", "status": 400}))

    with pytest.raises(ProblemError) as reported:
        _called(reaching, method, _arguments(method, optionals=False))

    assert reported.value.status == 400
    assert reported.value.problem is None


def test_a_failure_that_is_not_a_problem_document_is_still_reported(api: FakeHook0Api, generated: Any) -> None:
    reaching, method = _applications(generated, api.base_url)
    api.will_answer(ScriptedResponse(502, "a gateway wrote this, and it is not a problem document"))

    with pytest.raises(ProblemError) as reported:
        _called(reaching, method, _arguments(method, optionals=False))

    assert reported.value.status == 502
    assert reported.value.problem is None


def test_a_success_this_client_cannot_read_is_reported_rather_than_returned(api: FakeHook0Api, generated: Any) -> None:
    """A body the schema does not describe stops the read where it failed rather than being returned."""
    reaching, method = _applications(generated, api.base_url)
    api.will_answer(ScriptedResponse(200, {"a member": "the schema never declared"}))

    with pytest.raises(DecodeError):
        _called(reaching, method, _arguments(method, optionals=False))


def test_a_named_problem_reaches_a_caller_that_names_only_it(api: FakeHook0Api, generated: Any) -> None:
    """The narrowest catch a caller can write still catches what the API reported."""
    reaching, method = _applications(generated, api.base_url)
    api.will_answer(
        ScriptedResponse(
            404,
            {
                "id": "NotFound",
                "title": "Not found",
                "detail": "This application does not exist.",
                "status": 404,
                "type": "https://documentation.hook0.com/problems",
            },
        )
    )

    with pytest.raises(NotFoundError) as reported:
        _called(reaching, method, _arguments(method, optionals=False))

    assert reported.value.problem is not None
    assert reported.value.problem.detail == "This application does not exist."
