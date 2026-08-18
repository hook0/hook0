// What the generated request layer puts on the wire, and what it does with what comes back.
//
// The generated half is handed a transport and nothing else, so these cases hand it the real one and
// watch a real API answer: the path it interpolated, the query it assembled, the credential it
// carried, the type it read back, and the error value it answered when the answer was a problem.
//
// Nothing here names an operation the API does not declare. What is exercised is reached through the
// generated package, so an operation that stops being declared takes its case with it rather than
// leaving one that compiles against nothing.

package hook0_test

import (
	"bytes"
	"context"
	"encoding/json"
	"errors"
	"fmt"
	"go/ast"
	"go/parser"
	source "go/token"
	"net/url"
	"os"
	"path/filepath"
	"reflect"
	"slices"
	"strings"
	"testing"
	"time"

	hook0 "github.com/hook0/hook0-go"
	"github.com/hook0/hook0-go/generated"
)

func applications(api *fakeAPI) *generated.ApplicationsAPI {
	return generated.NewApplicationsAPI(
		hook0.NewTransport(api.baseURL(), token, 0, 0),
	)
}

func anApplication() map[string]any {
	return map[string]any{
		"application_id":  applicationId,
		"name":            "an application",
		"organization_id": organizationId,
		"consumption":     map[string]any{"events_per_day": 12},
		"onboarding_steps": map[string]any{
			"event":        "Done",
			"event_type":   "ToDo",
			"subscription": "ToDo",
		},
		"quotas": map[string]any{
			"days_of_events_retention_limit": 7,
			"events_per_day_limit":           100,
		},
	}
}

func aProblem() map[string]any {
	return map[string]any{
		"id":     "NotFound",
		"title":  "Not found",
		"detail": "This application does not exist.",
		"status": 404,
		"type":   "https://documentation.hook0.com/problems",
	}
}

func TestAGeneratedMethodReadsBackTheTypeTheAPIDeclares(t *testing.T) {
	api := listen(t)
	api.willAnswer(scriptedResponse{status: 200, body: anApplication()})

	application, err := applications(api).Get(bounded(t), applicationId)
	if err != nil {
		t.Fatalf("the operation failed: %v", err)
	}

	if application.ApplicationId != generated.UUID(applicationId) {
		t.Errorf("the application read back carries %q as its identifier", application.ApplicationId)
	}
	if application.Quotas.EventsPerDayLimit != 100 {
		t.Errorf("the quota read back is %d", application.Quotas.EventsPerDayLimit)
	}
	if application.OnboardingSteps.Event != generated.ApplicationInfoOnboardingStepsEventDone {
		t.Errorf("the onboarding step read back is %q", application.OnboardingSteps.Event)
	}
}

func TestAGeneratedMethodFillsThePathAndCarriesTheCredential(t *testing.T) {
	api := listen(t)
	api.willAnswer(scriptedResponse{status: 200, body: anApplication()})

	if _, err := applications(api).Get(bounded(t), applicationId); err != nil {
		t.Fatalf("the operation failed: %v", err)
	}

	request := api.requests()[0]
	if request.method != "GET" {
		t.Errorf("the operation was issued with %s", request.method)
	}
	// The identifier lands in the path rather than staying as the placeholder that named it.
	if request.target != "/api/v1/applications/"+applicationId {
		t.Errorf("the operation reached %q", request.target)
	}
	if carried := request.headers.Get("Authorization"); carried != "Bearer "+token {
		t.Errorf("the request carried %q as its credential", carried)
	}
}

func TestAGeneratedMethodAssemblesTheQueryTheOperationDeclares(t *testing.T) {
	api := listen(t)
	api.willAnswer(scriptedResponse{status: 200, body: []any{}})

	listed, err := applications(api).List(bounded(t), organizationId)
	if err != nil {
		t.Fatalf("the operation failed: %v", err)
	}

	if len(listed) != 0 {
		t.Errorf("an empty listing read back %d applications", len(listed))
	}
	if target := api.requests()[0].target; target != "/api/v1/applications/?organization_id="+organizationId {
		t.Errorf("the operation reached %q", target)
	}
}

func TestAGeneratedMethodReadsAListOfTheTypeTheAPIDeclares(t *testing.T) {
	api := listen(t)
	api.willAnswer(scriptedResponse{
		status: 200,
		body: []any{map[string]any{
			"application_id":  applicationId,
			"name":            "an application",
			"organization_id": organizationId,
		}},
	})

	listed, err := applications(api).List(bounded(t), organizationId)
	if err != nil {
		t.Fatalf("the operation failed: %v", err)
	}

	if len(listed) != 1 || listed[0].Name != "an application" {
		t.Errorf("the listing read back %v", listed)
	}
}

func TestAProblemTheAPIReportsIsAnsweredAsTheValueItNames(t *testing.T) {
	api := listen(t)
	api.willAnswer(scriptedResponse{status: 404, body: aProblem()})

	_, err := applications(api).Get(bounded(t), applicationId)
	if err == nil {
		t.Fatal("a problem the API reported was read as a success")
	}

	if !errors.Is(err, generated.ErrNotFound) {
		t.Errorf("the failure is %v, which is not the problem the API named", err)
	}
	if errors.Is(err, generated.ErrForbidden) {
		t.Error("the failure answers to a problem the API did not name")
	}

	var reported *generated.ProblemError
	if !errors.As(err, &reported) {
		t.Fatalf("the failure is %T, not the one every problem is a kind of", err)
	}
	if reported.Status != 404 {
		t.Errorf("the failure says the API answered %d", reported.Status)
	}
	if reported.Problem == nil || reported.Problem.Detail != "This application does not exist." {
		t.Errorf("the failure does not carry the document the API answered: %+v", reported.Problem)
	}
}

func TestAFailureThatIsNoProblemDocumentIsStillReported(t *testing.T) {
	api := listen(t)
	api.willAnswer(scriptedResponse{
		status: 502,
		body:   "a gateway wrote this, and it is not a problem document",
	})

	_, err := applications(api).Get(bounded(t), applicationId)
	if err == nil {
		t.Fatal("a gateway failure was read as a success")
	}

	var reported *generated.ProblemError
	if !errors.As(err, &reported) {
		t.Fatalf("the failure is %T, not the one every failure is a kind of", err)
	}
	if reported.Status != 502 {
		t.Errorf("the failure says the API answered %d", reported.Status)
	}
	if reported.Problem != nil {
		t.Errorf("a body naming no problem was read as one: %+v", reported.Problem)
	}
	if reported.Kind != "" {
		t.Errorf("a body naming no problem was given the kind %q", reported.Kind)
	}
}

func TestAnIdentifierThatIsNotOneIsRefusedWhileItIsRead(t *testing.T) {
	api := listen(t)
	answered := anApplication()
	answered["application_id"] = "not-an-identifier"
	api.willAnswer(scriptedResponse{status: 200, body: answered})

	if _, err := applications(api).Get(bounded(t), applicationId); err == nil {
		t.Fatal("a document whose identifier is not one was read as if it were")
	}
}

func TestAGeneratedGroupIsBuiltOnTheClientsOwnTransport(t *testing.T) {
	api := listen(t)
	api.willAnswer(scriptedResponse{status: 200, body: anApplication()})

	// The seam is structural: the hand-written transport answers to the interface the generated
	// package declares without either half naming the other.
	group := generated.NewApplicationsAPI(client(api, promptOptions(1)).Transport())

	if _, err := group.Get(bounded(t), applicationId); err != nil {
		t.Fatalf("a group built on the client's own transport failed: %v", err)
	}
}

// What follows walks the whole of the generated package rather than one group of it.
//
// The groups are the one thing Go cannot be asked for: a package's types are not enumerable at run
// time, and the struct behind a group holds an unexported field, so the constructors below are the
// only door in. They are therefore written out — and then held to what the generated source
// actually declares, so that a group the API grows cannot quietly go unreached. Everything past
// that door is discovered: the methods of a group, the arguments each one takes, and the type each
// one reads back all come off the method itself.

// aString is what every string-shaped argument is given. It carries the two characters a path
// segment may not leave as they are, so a value reaching a path proves it was escaped rather than
// pasted.
const aString = "a value/with a space"

// maxDepth bounds how deep a value is built. No schema the API declares nests anywhere near this
// far, and the bound turns a document that describes itself into a failure rather than a recursion
// that never returns.
const maxDepth = 8

// maxAncestors bounds the walk up to the repository holding the API document.
const maxAncestors = 8

// publicTag marks an operation as part of the surface an SDK exposes. A document marking none of
// its operations with it declares the whole of itself public, which is what the generator does with
// the tag and therefore what this suite holds a target to.
const publicTag = "public"

// aMoment is what every instant-shaped member is given: a fixed one, so that what is written back
// is compared against what was sent rather than against the clock.
var aMoment = time.Date(2026, time.January, 2, 3, 4, 5, 0, time.UTC)

// verbs are the methods a request line can carry, which is what tells an operation apart from the
// rest of what a path item holds.
var verbs = []string{"get", "put", "post", "delete", "options", "head", "patch", "trace"}

// generatedGroups is every group of operations the package declares, each one reaching the same API.
func generatedGroups(transport generated.Transport) []any {
	return []any{
		generated.NewApplicationSecretsAPI(transport),
		generated.NewApplicationsAPI(transport),
		generated.NewErrorsAPI(transport),
		generated.NewEventTypesAPI(transport),
		generated.NewEventsAPI(transport),
		generated.NewEventsPerDayAPI(transport),
		generated.NewInstanceAPI(transport),
		generated.NewPayloadContentTypesAPI(transport),
		generated.NewQuotasAPI(transport),
		generated.NewRequestAttemptsAPI(transport),
		generated.NewResponseAPI(transport),
		generated.NewServiceTokenAPI(transport),
		generated.NewSubscriptionsAPI(transport),
	}
}

// declaredGroups is every group the generated source declares a constructor for.
func declaredGroups(t *testing.T) map[string]bool {
	t.Helper()

	parsed, err := parser.ParseFile(source.NewFileSet(), filepath.Join("generated", "api.go"), nil, 0)
	if err != nil {
		t.Fatalf("the generated source could not be read: %v", err)
	}

	found := map[string]bool{}
	for _, declaration := range parsed.Decls {
		function, ok := declaration.(*ast.FuncDecl)
		if !ok || function.Recv != nil {
			continue
		}
		name := function.Name.Name
		if strings.HasPrefix(name, "New") && strings.HasSuffix(name, "API") {
			found[strings.TrimPrefix(name, "New")] = true
		}
	}
	if len(found) == 0 {
		t.Fatal("the generated source declares no group of operations at all")
	}
	return found
}

// declaredOperation is one operation the API document declares, as a request has to look to be it.
type declaredOperation struct {
	method        string
	template      string
	requiredQuery map[string]bool
	optionalQuery map[string]bool
}

// wantedQuery is the parameters a request carries, given whether the optional ones were asked for.
func (o declaredOperation) wantedQuery(optionals bool) map[string]bool {
	wanted := map[string]bool{}
	for name := range o.requiredQuery {
		wanted[name] = true
	}
	if optionals {
		for name := range o.optionalQuery {
			wanted[name] = true
		}
	}
	return wanted
}

// matches reports whether a request line landed on this operation.
func (o declaredOperation) matches(target string) bool {
	path, _, _ := strings.Cut(target, "?")
	wanted := strings.Split(o.template, "/")
	got := strings.Split(path, "/")
	if len(wanted) != len(got) {
		return false
	}
	for index, declared := range wanted {
		if strings.HasPrefix(declared, "{") && strings.HasSuffix(declared, "}") {
			// A parameter stands for a segment that is there; an empty one is the trailing slash of
			// another path rather than a value.
			if got[index] == "" {
				return false
			}
			continue
		}
		if declared != got[index] {
			return false
		}
	}
	return true
}

// apiDocument is the OpenAPI document the generator was run against, out of the repository holding
// it.
func apiDocument(t *testing.T) []byte {
	t.Helper()

	at := "."
	for range maxAncestors {
		read, err := os.ReadFile(filepath.Join(at, "api", "openapi.snapshot.json"))
		if err == nil {
			return read
		}
		at = filepath.Join(at, "..")
	}
	t.Fatalf("no `api/openapi.snapshot.json` within %d directories of the module", maxAncestors)
	return nil
}

// declaredOperations is every operation an SDK is built out of, which is what the document marks as
// public.
func declaredOperations(t *testing.T) []declaredOperation {
	t.Helper()

	var document struct {
		Paths map[string]map[string]json.RawMessage `json:"paths"`
	}
	if err := json.Unmarshal(apiDocument(t), &document); err != nil {
		t.Fatalf("the API document could not be read: %v", err)
	}

	type specOperation struct {
		Tags       []string `json:"tags"`
		Parameters []struct {
			Name     string `json:"name"`
			In       string `json:"in"`
			Required bool   `json:"required"`
		} `json:"parameters"`
	}

	var public, all []declaredOperation
	for template, item := range document.Paths {
		for _, verb := range verbs {
			written, declared := item[verb]
			if !declared {
				continue
			}

			var operation specOperation
			if err := json.Unmarshal(written, &operation); err != nil {
				t.Fatalf("`%s %s` could not be read: %v", verb, template, err)
			}

			read := declaredOperation{
				method:        strings.ToUpper(verb),
				template:      template,
				requiredQuery: map[string]bool{},
				optionalQuery: map[string]bool{},
			}
			for _, parameter := range operation.Parameters {
				if parameter.In != "query" {
					continue
				}
				if parameter.Required {
					read.requiredQuery[parameter.Name] = true
				} else {
					read.optionalQuery[parameter.Name] = true
				}
			}

			all = append(all, read)
			if slices.Contains(operation.Tags, publicTag) {
				public = append(public, read)
			}
		}
	}

	if len(all) == 0 {
		t.Fatal("the API document declares no operation at all")
	}
	if len(public) > 0 {
		return public
	}
	return all
}

// unmarshaler is the shape a type has when it reads itself, which is how a scalar the standard
// library has no type for refuses text that does not spell one.
var unmarshaler = reflect.TypeOf((*json.Unmarshaler)(nil)).Elem()

// populate fills a value with something of every type it is made of, so that what is written back
// can be compared against what was sent member by member rather than against a zero.
//
// With optionals off, everything the document lets the API leave out is left out, which is what
// exercises the other side of every branch that reads a member that may be absent.
func populate(t *testing.T, value reflect.Value, optionals bool, depth int) {
	t.Helper()

	if depth > maxDepth {
		t.Fatalf("%s nests more than %d deep", value.Type(), maxDepth)
	}

	switch value.Kind() {
	case reflect.Pointer:
		if !optionals {
			return
		}
		value.Set(reflect.New(value.Type().Elem()))
		populate(t, value.Elem(), optionals, depth+1)
	case reflect.Struct:
		if value.Type() == reflect.TypeOf(time.Time{}) {
			value.Set(reflect.ValueOf(aMoment))
			return
		}
		for index := range value.NumField() {
			if value.Type().Field(index).IsExported() {
				populate(t, value.Field(index), optionals, depth+1)
			}
		}
	case reflect.String:
		// A type that reads itself is one the standard library has no notion of — an identifier, a
		// day — and the only text every one of them accepts is its own zero, which is what it
		// already holds.
		if !reflect.PointerTo(value.Type()).Implements(unmarshaler) {
			value.SetString("a value the API answered")
		}
	case reflect.Bool:
		value.SetBool(true)
	case reflect.Int, reflect.Int8, reflect.Int16, reflect.Int32, reflect.Int64:
		value.SetInt(12)
	case reflect.Uint, reflect.Uint8, reflect.Uint16, reflect.Uint32, reflect.Uint64:
		value.SetUint(12)
	case reflect.Float32, reflect.Float64:
		value.SetFloat(1.5)
	case reflect.Slice:
		item := reflect.New(value.Type().Elem()).Elem()
		populate(t, item, optionals, depth+1)
		value.Set(reflect.Append(reflect.MakeSlice(value.Type(), 0, 1), item))
	case reflect.Map:
		item := reflect.New(value.Type().Elem()).Elem()
		populate(t, item, optionals, depth+1)
		key := reflect.New(value.Type().Key()).Elem()
		populate(t, key, optionals, depth+1)
		made := reflect.MakeMap(value.Type())
		made.SetMapIndex(key, item)
		value.Set(made)
	case reflect.Interface:
		// A member the document does not describe, kept as it arrived whatever it is.
		value.Set(reflect.ValueOf("a value the document describes nothing about"))
	default:
		t.Fatalf("the generated package carries a %s nothing here knows how to build", value.Kind())
	}
}

// answerFor is the document the API answers one operation with, out of the type that operation
// reads back.
func answerFor(t *testing.T, method reflect.Type, optionals bool) any {
	t.Helper()

	if method.NumOut() == 1 {
		// Nothing but the failure: the operation reads no value of its own.
		return map[string]any{}
	}

	read := method.Out(0)
	if read.Kind() == reflect.Pointer {
		read = read.Elem()
	}
	answered := reflect.New(read).Elem()
	populate(t, answered, optionals, 0)
	return answered.Interface()
}

// argumentsFor is what one operation is asked with: everything it requires, and what it does not as
// asked for or left out.
func argumentsFor(t *testing.T, method reflect.Value, ctx context.Context, optionals bool) []reflect.Value {
	t.Helper()

	shape := method.Type()
	given := make([]reflect.Value, 0, shape.NumIn())
	for index := range shape.NumIn() {
		wanted := shape.In(index)
		switch {
		case index == 0:
			given = append(given, reflect.ValueOf(ctx))
		case wanted.Kind() == reflect.Pointer && wanted.Elem().Kind() == reflect.String:
			// A parameter the operation does not require, which the API is asked without when the
			// case is about what a request carries when it is left out.
			if !optionals {
				given = append(given, reflect.Zero(wanted))
				continue
			}
			held := reflect.New(wanted.Elem())
			held.Elem().SetString(aString)
			given = append(given, held)
		case wanted.Kind() == reflect.String:
			given = append(given, reflect.ValueOf(aString).Convert(wanted))
		case wanted.Kind() == reflect.Struct:
			// The body the operation reads, filled the same way an answer is.
			body := reflect.New(wanted).Elem()
			populate(t, body, optionals, 0)
			given = append(given, body)
		default:
			t.Fatalf("an operation takes a %s nothing here knows how to give", wanted)
		}
	}
	return given
}

func TestEveryGroupTheGeneratedSourceDeclaresIsReached(t *testing.T) {
	built := map[string]bool{}
	for _, group := range generatedGroups(hook0.NewTransport("http://127.0.0.1:1", token, 0, 0)) {
		built[reflect.TypeOf(group).Elem().Name()] = true
	}

	declared := declaredGroups(t)
	for name := range declared {
		if !built[name] {
			t.Errorf("the generated package declares `%s` and nothing below reaches it", name)
		}
	}
	for name := range built {
		if !declared[name] {
			t.Errorf("`%s` is reached below and the generated package no longer declares it", name)
		}
	}
}

func TestEveryOperationTheDocumentDeclaresIsReachedTheWayItDeclaresIt(t *testing.T) {
	for _, optionals := range []bool{true, false} {
		t.Run(map[bool]string{true: "with-optional-arguments", false: "without-them"}[optionals], func(t *testing.T) {
			api := listen(t)
			transport := hook0.NewTransport(api.baseURL(), token, 0, 0)
			operations := declaredOperations(t)
			reached := map[string]bool{}

			for _, group := range generatedGroups(transport) {
				reaching := reflect.ValueOf(group)
				for index := range reaching.NumMethod() {
					named := reflect.TypeOf(group).Elem().Name() + "." + reaching.Type().Method(index).Name
					method := reaching.Method(index)

					answered := answerFor(t, method.Type(), optionals)
					api.willAnswer(scriptedResponse{status: 200, body: answered})

					out := method.Call(argumentsFor(t, method, bounded(t), optionals))
					if failure := out[len(out)-1]; !failure.IsNil() {
						t.Fatalf("%s failed: %v", named, failure.Interface())
					}

					request := api.requests()[api.requestCount()-1]
					operation := operationOf(t, named, operations, request)
					reached[operation.method+" "+operation.template] = true

					assertRequestIsWhatTheDocumentDeclares(t, named, operation, request, optionals)
					assertAnswerIsWrittenBackAsItArrived(t, named, out, answered)
				}
			}

			for _, operation := range operations {
				if !reached[operation.method+" "+operation.template] {
					t.Errorf("`%s %s` is declared and no generated method reaches it", operation.method, operation.template)
				}
			}
		})
	}
}

// operationOf is which operation of the document a request landed on.
func operationOf(t *testing.T, named string, operations []declaredOperation, request receivedRequest) declaredOperation {
	t.Helper()

	var matched []declaredOperation
	for _, operation := range operations {
		if operation.method == request.method && operation.matches(request.target) {
			matched = append(matched, operation)
		}
	}
	if len(matched) != 1 {
		t.Fatalf("%s reached `%s %s`, which is %d of the operations declared", named, request.method, request.target, len(matched))
	}
	return matched[0]
}

func assertRequestIsWhatTheDocumentDeclares(
	t *testing.T,
	named string,
	operation declaredOperation,
	request receivedRequest,
	optionals bool,
) {
	t.Helper()

	if carried := request.headers.Get("Authorization"); carried != "Bearer "+token {
		t.Errorf("%s carried %q as its credential", named, carried)
	}

	path, _, _ := strings.Cut(request.target, "?")
	for index, declared := range strings.Split(operation.template, "/") {
		if !strings.HasPrefix(declared, "{") {
			continue
		}
		// The value lands in the path escaped, so that nothing in it can name a segment the
		// operation never had.
		if got := strings.Split(path, "/")[index]; got != url.PathEscape(aString) {
			t.Errorf("%s left `%s` as %q", named, declared, got)
		}
	}

	_, written, _ := strings.Cut(request.target, "?")
	carried, err := url.ParseQuery(written)
	if err != nil {
		t.Fatalf("%s assembled a query that does not read back: %v", named, err)
	}
	wanted := operation.wantedQuery(optionals)
	if len(carried) != len(wanted) {
		t.Errorf("%s carried %v where the document declares %v", named, carried, wanted)
	}
	for name := range wanted {
		if got := carried.Get(name); got != aString {
			t.Errorf("%s carried `%s` as %q", named, name, got)
		}
	}
}

func assertAnswerIsWrittenBackAsItArrived(t *testing.T, named string, out []reflect.Value, answered any) {
	t.Helper()

	if len(out) == 1 {
		return
	}

	sent, err := json.Marshal(answered)
	if err != nil {
		t.Fatalf("%s: the answer the case scripted could not be written: %v", named, err)
	}
	read, err := json.Marshal(out[0].Interface())
	if err != nil {
		t.Fatalf("%s: what it read back could not be written: %v", named, err)
	}
	if !bytes.Equal(sent, read) {
		t.Errorf("%s read back %s where the API answered %s", named, read, sent)
	}
}

// declaredProblems is every problem the API document says it can report, out of the schema of the
// body every failure carries.
func declaredProblems(t *testing.T) []string {
	t.Helper()

	var document struct {
		Components struct {
			Schemas struct {
				Problem struct {
					Properties struct {
						Id struct {
							Enum []string `json:"enum"`
						} `json:"id"`
					} `json:"properties"`
				} `json:"Problem"`
			} `json:"schemas"`
		} `json:"components"`
	}
	if err := json.Unmarshal(apiDocument(t), &document); err != nil {
		t.Fatalf("the API document could not be read: %v", err)
	}

	named := document.Components.Schemas.Problem.Properties.Id.Enum
	if len(named) == 0 {
		t.Fatal("the API document names no problem at all")
	}
	return named
}

func TestEveryProblemTheDocumentNamesIsAnsweredAsItsOwnFailure(t *testing.T) {
	api := listen(t)
	group := applications(api)

	for _, problem := range declaredProblems(t) {
		api.willAnswer(scriptedResponse{status: 400, body: map[string]any{
			"id":     problem,
			"status": 400,
			"title":  "refused",
			"detail": "what the case scripted",
			"type":   "https://hook0.com/documentation/errors/" + problem,
		}})

		_, err := group.Get(bounded(t), applicationId)
		if err == nil {
			t.Fatalf("`%s` was read as a success", problem)
		}

		var reported *generated.ProblemError
		if !errors.As(err, &reported) {
			t.Fatalf("`%s` is %T, not the one every problem is a kind of", problem, err)
		}
		if string(reported.Kind) != problem {
			t.Errorf("`%s` was answered as `%s`", problem, reported.Kind)
		}
		if reported.Status != 400 {
			t.Errorf("`%s` says the API answered %d", problem, reported.Status)
		}
		if reported.Problem == nil || reported.Problem.Detail != "what the case scripted" {
			t.Errorf("`%s` does not carry the document the API answered", problem)
		}
		if said := reported.Error(); said == "" {
			t.Errorf("`%s` says nothing about itself", problem)
		}
	}
}

func TestEveryProblemTheDocumentNamesIsDeclaredAsAValueACallerCanCompareAgainst(t *testing.T) {
	parsed, err := parser.ParseFile(source.NewFileSet(), filepath.Join("generated", "errors.go"), nil, 0)
	if err != nil {
		t.Fatalf("the generated source could not be read: %v", err)
	}

	sentinels := map[string]bool{}
	for _, declaration := range parsed.Decls {
		general, ok := declaration.(*ast.GenDecl)
		if !ok {
			continue
		}
		for _, spec := range general.Specs {
			value, ok := spec.(*ast.ValueSpec)
			if !ok {
				continue
			}
			for _, name := range value.Names {
				if strings.HasPrefix(name.Name, "Err") {
					sentinels[strings.TrimPrefix(name.Name, "Err")] = true
				}
			}
		}
	}

	for _, problem := range declaredProblems(t) {
		if !sentinels[problem] {
			t.Errorf("the document names `%s` and the generated package declares no value for it", problem)
		}
		delete(sentinels, problem)
	}
	for left := range sentinels {
		t.Errorf("the generated package declares a value for `%s`, which the document no longer names", left)
	}
}

func TestAProblemValueSaysWhichProblemItIsOnItsOwn(t *testing.T) {
	// Read on its own, away from any answer, a value still names the problem it stands for.
	if said := generated.ErrNotFound.Error(); said != "NotFound" {
		t.Errorf("the value declared for `NotFound` reads as %q", said)
	}
}

// scalarTexts are the shapes a scalar the standard library has no type for is spelled in, and one
// that spells none of them. Every such type accepts its own zero and at least one of these, and
// refuses at least one, whichever it is.
var scalarTexts = []string{
	"3f2504e0-4f89-41d3-9a0c-0305e82c3301",
	"2026-01-02",
	"neither one nor the other",
	// As long as one of the shapes above and wrong in one place, which is where a reader that
	// only measures what it was given stops telling one from the other.
	"3f2504e0-4f89-41d3-9a0c-0305e82c330g",
	"3f2504e0x4f89-41d3-9a0c-0305e82c3301",
	"2026-13-02",
}

// scalarsReachedFrom collects every type a generated method reads back, and every type reachable
// from one, that reads itself out of a document.
func scalarsReachedFrom(t *testing.T, from reflect.Type, found map[reflect.Type]bool, seen map[reflect.Type]bool, depth int) {
	t.Helper()

	if depth > maxDepth || seen[from] {
		return
	}
	seen[from] = true

	if from.Kind() == reflect.String && reflect.PointerTo(from).Implements(unmarshaler) {
		found[from] = true
		return
	}

	switch from.Kind() {
	case reflect.Pointer, reflect.Slice, reflect.Array, reflect.Map:
		scalarsReachedFrom(t, from.Elem(), found, seen, depth+1)
	case reflect.Struct:
		for index := range from.NumField() {
			if from.Field(index).IsExported() {
				scalarsReachedFrom(t, from.Field(index).Type, found, seen, depth+1)
			}
		}
	}
}

func TestEveryScalarTheStandardLibraryHasNoTypeForReadsOnlyWhatItSpells(t *testing.T) {
	found := map[reflect.Type]bool{}
	seen := map[reflect.Type]bool{}
	for _, group := range generatedGroups(hook0.NewTransport("http://127.0.0.1:1", token, 0, 0)) {
		reaching := reflect.ValueOf(group)
		for index := range reaching.NumMethod() {
			shape := reaching.Method(index).Type()
			if shape.NumOut() > 1 {
				scalarsReachedFrom(t, shape.Out(0), found, seen, 0)
			}
		}
	}
	if len(found) == 0 {
		t.Fatal("the generated package carries no scalar of its own")
	}

	for scalar := range found {
		accepted, refused := 0, 0
		for _, text := range scalarTexts {
			written, err := json.Marshal(text)
			if err != nil {
				t.Fatalf("%s: %v", scalar, err)
			}

			read := reflect.New(scalar)
			if err := json.Unmarshal(written, read.Interface()); err != nil {
				refused++
				continue
			}
			accepted++
			// What it read is the text it was written as, which is what lets a document be read
			// and written back unchanged.
			if said := read.Elem().Interface().(fmt.Stringer).String(); said != text {
				t.Errorf("%s read %q back as %q", scalar, text, said)
			}
		}

		if accepted == 0 {
			t.Errorf("%s spells none of %v, and nothing here knows what it does spell", scalar, scalarTexts)
		}
		if refused == 0 {
			t.Errorf("%s accepted every text offered to it, so it describes no shape at all", scalar)
		}

		// Its own zero is what a member the document requires and the API did not answer reads as,
		// and a type refusing a value its own decoder produces could not write back what it read.
		own := reflect.New(scalar)
		if err := json.Unmarshal([]byte(`""`), own.Interface()); err != nil {
			t.Errorf("%s refuses its own zero: %v", scalar, err)
		}
		if said := own.Elem().Interface().(fmt.Stringer).String(); said != "" {
			t.Errorf("%s reads its own zero back as %q", scalar, said)
		}
		if err := json.Unmarshal([]byte(`12`), reflect.New(scalar).Interface()); err == nil {
			t.Errorf("%s read a number as text", scalar)
		}
	}
}

func TestEveryOperationAnswersTheProblemTheAPIReportedRatherThanAValue(t *testing.T) {
	api := listen(t)
	transport := hook0.NewTransport(api.baseURL(), token, 0, 0)

	for _, group := range generatedGroups(transport) {
		reaching := reflect.ValueOf(group)
		for index := range reaching.NumMethod() {
			named := reflect.TypeOf(group).Elem().Name() + "." + reaching.Type().Method(index).Name
			method := reaching.Method(index)
			api.willAnswer(scriptedResponse{status: 404, body: aProblem()})

			out := method.Call(argumentsFor(t, method, bounded(t), false))

			failure, _ := out[len(out)-1].Interface().(error)
			if failure == nil {
				t.Fatalf("%s read a problem the API reported as a success", named)
			}
			if !errors.Is(failure, generated.ErrNotFound) {
				t.Errorf("%s answered %v, which is not the problem the API named", named, failure)
			}
			if len(out) > 1 && !out[0].IsZero() {
				t.Errorf("%s answered a value beside the problem: %v", named, out[0].Interface())
			}
		}
	}
}

func TestEveryOperationAnswersAnAnswerItCannotReadRatherThanAValue(t *testing.T) {
	api := listen(t)
	transport := hook0.NewTransport(api.baseURL(), token, 0, 0)

	for _, group := range generatedGroups(transport) {
		reaching := reflect.ValueOf(group)
		for index := range reaching.NumMethod() {
			named := reflect.TypeOf(group).Elem().Name() + "." + reaching.Type().Method(index).Name
			method := reaching.Method(index)
			if method.Type().NumOut() == 1 {
				// Nothing is read out of a success, so there is nothing here that could be unreadable.
				continue
			}

			// Neither the object nor the array any operation reads back, whichever it reads.
			api.willAnswer(scriptedResponse{status: 200, body: "a gateway wrote this"})

			out := method.Call(argumentsFor(t, method, bounded(t), false))

			failure, _ := out[len(out)-1].Interface().(error)
			if failure == nil {
				t.Fatalf("%s read a body the document does not describe as if it did", named)
			}
			if !strings.Contains(failure.Error(), "cannot read") {
				t.Errorf("%s said %q about a body it could not read", named, failure)
			}
		}
	}
}

func TestNoOperationAnswersAValueWhenTheAPIWasNeverReached(t *testing.T) {
	unreachable := hook0.NewTransport("http://127.0.0.1:1", token, 0, 0)

	for _, group := range generatedGroups(unreachable) {
		reaching := reflect.ValueOf(group)
		for index := range reaching.NumMethod() {
			named := reflect.TypeOf(group).Elem().Name() + "." + reaching.Type().Method(index).Name
			method := reaching.Method(index)

			out := method.Call(argumentsFor(t, method, bounded(t), false))

			failure, _ := out[len(out)-1].Interface().(error)
			if failure == nil {
				t.Fatalf("%s answered a success although nothing was listening", named)
			}
			if len(out) > 1 && !out[0].IsZero() {
				t.Errorf("%s answered a value beside the failure: %v", named, out[0].Interface())
			}
		}
	}
}
