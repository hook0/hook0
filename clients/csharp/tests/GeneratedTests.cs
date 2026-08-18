// What the generator wrote, exercised as a caller would use it.
//
// Nothing here lists a type, a member or an operation: the surface is found by looking at what was
// emitted, so a schema the document adds joins this suite the moment the generated files carry it,
// and one it drops stops being asserted about rather than being asserted about twice.
//
// The two flavours are held against each other rather than each against a list. A blocking group and
// a Task-returning one that declared different operations would be two SDKs sharing a package, and
// the reason for writing them from one description is exactly that they cannot.

using System;
using System.Collections.Generic;
using System.Globalization;
using System.Linq;
using System.Reflection;
using System.Runtime.CompilerServices;
using System.Text;
using System.Text.Json;
using System.Text.Json.Nodes;
using System.Threading;
using System.Threading.Tasks;
using Hook0.Generated;
using Xunit;

namespace Hook0.Tests;

/// <summary>What the generated half declares, and what it does over a real socket.</summary>
public sealed class GeneratedTests : ApiCase
{
    /// <summary>Everything the generator wrote, found rather than listed.</summary>
    private static readonly IReadOnlyList<Type> Declared =
    [
        .. typeof(ProblemException).Assembly
            .GetExportedTypes()
            .Where(declared => declared.Namespace == typeof(ProblemException).Namespace)
            .OrderBy(declared => declared.Name, StringComparer.Ordinal),
    ];

    /// <summary>The names every object already answers to, which no member may replace.</summary>
    private static readonly IReadOnlySet<string> Shadowed = new HashSet<string>(StringComparer.Ordinal)
    {
        "Equals",
        "GetHashCode",
        "GetType",
        "MemberwiseClone",
        "ReferenceEquals",
        "ToString",
    };

    /// <summary>What a Task-returning method of the generated surface is named with.</summary>
    private const string AsyncSuffix = "Async";

    /// <summary>The tag the API marks the operations an SDK is written for with.</summary>
    private const string PublicTag = "public";

    /// <summary>How far a value is built into itself before this suite reads it as one it cannot build.</summary>
    private const int MaxNesting = 16;

    [Fact]
    public void TheGeneratorWroteASurfaceAtAll()
    {
        Assert.NotEmpty(Declared);
        Assert.Contains(Declared, declared => declared.Name.EndsWith("Api", StringComparison.Ordinal));
    }

    [Fact]
    public void NoGeneratedMemberReplacesOneEveryObjectAlreadyAnswersTo()
    {
        // A property spelled `Equals`, `GetHashCode` or `ToString` does not compile; one spelled
        // `GetType`, `MemberwiseClone` or `ReferenceEquals` hides the inherited member and is a
        // warning this build treats as an error. Either way the emitter has to have spelled it out
        // of the way, and this is what says it did rather than that it happened not to have to.
        List<string> replaced =
        [
            .. from declared in Declared
               from member in Written(declared)
               where Shadowed.Contains(member.Name)
               select $"{declared.Name}.{member.Name}",
        ];

        Assert.True(replaced.Count == 0, $"the generated surface replaces: {string.Join(", ", replaced)}");
    }

    [Fact]
    public void NoGeneratedMemberCarriesTheNameOfTheTypeItSitsIn()
    {
        // C# refuses one outright, and it is not a vocabulary: it is one word per declaration.
        List<string> shadowing =
        [
            .. from declared in Declared
               where !declared.IsEnum
               from member in Written(declared)
               where member.Name == declared.Name
               select $"{declared.Name}.{member.Name}",
        ];

        Assert.True(shadowing.Count == 0, $"the generated surface declares: {string.Join(", ", shadowing)}");
    }

    [Fact]
    public void EveryOperationIsDeclaredInBothFlavours()
    {
        List<Type> blocking =
        [
            .. Declared.Where(declared =>
                declared.Name.EndsWith("Api", StringComparison.Ordinal)
                && !declared.Name.EndsWith("AsyncApi", StringComparison.Ordinal)),
        ];
        Assert.NotEmpty(blocking);

        foreach (Type group in blocking)
        {
            string awaiting = group.Name.Replace("Api", "AsyncApi", StringComparison.Ordinal);
            Type? mirrored = Declared.FirstOrDefault(declared => declared.Name == awaiting);
            Assert.True(mirrored is not null, $"`{group.Name}` has no `{awaiting}` beside it");

            IReadOnlySet<string> waits = Operations(group);
            IReadOnlySet<string> awaits =
                new HashSet<string>(Operations(mirrored!).Select(Shortened), StringComparer.Ordinal);

            Assert.True(
                waits.SetEquals(awaits),
                $"`{group.Name}` and `{awaiting}` declare different operations: " +
                $"{string.Join(", ", waits.Except(awaits).Concat(awaits.Except(waits)))}");
        }
    }

    [Fact]
    public void EveryTaskReturningMethodIsNamedAndCancellableAsTheEcosystemExpects()
    {
        foreach (Type group in Declared.Where(declared =>
            declared.Name.EndsWith("AsyncApi", StringComparison.Ordinal)))
        {
            foreach (MethodInfo method in Methods(group))
            {
                Assert.True(
                    method.Name.EndsWith(AsyncSuffix, StringComparison.Ordinal),
                    $"`{group.Name}.{method.Name}` returns a Task and is not named for it");
                Assert.True(
                    typeof(Task).IsAssignableFrom(method.ReturnType),
                    $"`{group.Name}.{method.Name}` is named for a Task and returns none");

                ParameterInfo last = method.GetParameters()[^1];
                Assert.True(
                    last.ParameterType == typeof(CancellationToken) && last.HasDefaultValue,
                    $"`{group.Name}.{method.Name}` cannot be abandoned");
            }
        }
    }

    [Fact]
    public void EveryClosedListCarriesTheStringsTheApiAnswers()
    {
        // The constants are the wire values themselves, so a name moved out of the way of the
        // language never reaches the wire. That is what this asserts: every constant's value is a
        // value the list declares, and the list answers to each of them.
        List<Type> lists = [.. Declared.Where(declared => declared.GetProperty("Values") is not null)];
        Assert.NotEmpty(lists);

        foreach (Type list in lists)
        {
            IReadOnlyList<string> values =
                (IReadOnlyList<string>)list.GetProperty("Values")!.GetValue(null)!;
            MethodInfo carries = list.GetMethod("Contains")!;

            List<string> constants =
            [
                .. list.GetFields(BindingFlags.Public | BindingFlags.Static | BindingFlags.DeclaredOnly)
                    .Where(field => field.IsLiteral)
                    .Select(field => (string)field.GetRawConstantValue()!),
            ];

            Assert.Equal(values.OrderBy(value => value, StringComparer.Ordinal), constants.OrderBy(value => value, StringComparer.Ordinal));
            foreach (string value in values)
            {
                Assert.True((bool)carries.Invoke(null, [value])!, $"`{list.Name}` does not carry `{value}`");
            }

            Assert.False((bool)carries.Invoke(null, ["a value the API never declared"])!);
        }
    }

    [Fact]
    public void EveryProblemTheCatalogueNamesIsRaisedAsItsOwnFailure()
    {
        foreach (string problem in ProblemId.Values)
        {
            byte[] answered = Encoding.UTF8.GetBytes(
                new JsonObject
                {
                    ["id"] = problem,
                    ["status"] = 400,
                    ["title"] = "refused",
                    ["detail"] = "what the case scripted",
                    ["type"] = $"https://hook0.com/documentation/errors/{problem}",
                }.ToJsonString());

            ProblemException raised = Assert.Throws(
                Declared.First(declared => declared.Name == $"{problem}Exception"),
                () => Problems.RaiseForStatus(400, answered)) as ProblemException
                ?? throw new InvalidOperationException($"`{problem}` was not raised as a problem");

            Assert.Equal(400, raised.Status);
            Assert.Equal(problem, raised.Problem!.Id);
        }
    }

    [Fact]
    public void AProblemTheApiGrewAfterThisPackageIsStillReportedAsTheProblemItNames()
    {
        // The catalogue is what this package was generated from, and the API outlives the package it
        // was generated into. A problem it grows still arrives as a whole problem document, and a
        // caller has to be handed it — its status, and what it said — rather than a failure that
        // dropped everything the API took the trouble to say.
        byte[] answered = Encoding.UTF8.GetBytes(
            new JsonObject
            {
                ["id"] = "AProblemThisClientHasNeverHeardOf",
                ["status"] = 418,
                ["title"] = "refused",
                ["detail"] = "what a later API said",
                ["type"] = "https://hook0.com/documentation/errors/AProblemThisClientHasNeverHeardOf",
            }.ToJsonString());

        ProblemException raised = Assert.Throws<ProblemException>(() => Problems.RaiseForStatus(418, answered));

        Assert.Equal(418, raised.Status);
        Assert.Equal("AProblemThisClientHasNeverHeardOf", raised.Problem!.Id);
        Assert.Contains("what a later API said", raised.Message, StringComparison.Ordinal);
    }

    [Fact]
    public void AValueTravelsInARequestLineTheWayTheApiReadsOne()
    {
        // What a generated operation hands the transport for a path segment or a query value. The
        // spellings are the API's, not the platform's: a boolean is a word, a moment is RFC 3339, a
        // number carries no thousands separator and no locale's decimal mark.
        Assert.Equal(string.Empty, Runtime.Written(null));
        Assert.Equal("what the caller passed", Runtime.Written("what the caller passed"));
        Assert.Equal("true", Runtime.Written(true));
        Assert.Equal("false", Runtime.Written(false));
        Assert.Equal("7", Runtime.Written(7));
        Assert.Equal("1.5", Runtime.Written(1.5d));
        Assert.Equal("1.5", Runtime.Written(1.5f));
        Assert.Equal("2026-01-02", Runtime.Written(new DateOnly(2026, 1, 2)));
        Assert.Equal(
            "00000000-0000-4000-8000-000000000001",
            Runtime.Written(Guid.Parse("00000000-0000-4000-8000-000000000001")));
        Assert.Equal(
            "2026-01-02T03:04:05+00:00",
            Runtime.Written(new DateTimeOffset(2026, 1, 2, 3, 4, 5, TimeSpan.Zero)));
        Assert.Equal(
            "2026-01-02T03:04:05Z",
            Runtime.Written(new DateTime(2026, 1, 2, 3, 4, 5, DateTimeKind.Utc)));

        // A number is asserted by the promise rather than by the spelling: whatever text it travels
        // as reads back as the same number under a culture that is not the machine's, across the
        // range rather than at the one value a case picked. `1.5` above is the spelling; this is
        // what would still hold if a runtime shortened `1E+21`.
        foreach (double number in new[] { 1.5d, -1.5d, 0d, 0.1d, 1e21d, double.Epsilon, double.MaxValue })
        {
            string written = Runtime.Written(number);

            Assert.Equal(number, double.Parse(written, NumberStyles.Float, CultureInfo.InvariantCulture));
            Assert.DoesNotContain(",", written, StringComparison.Ordinal);
        }

        foreach (float number in new[] { 1.5f, -1.5f, 0f })
        {
            Assert.Equal(
                number,
                float.Parse(Runtime.Written(number), NumberStyles.Float, CultureInfo.InvariantCulture));
        }

        // A day, on the other hand, is parsed by the API, so its spelling is the contract — at the
        // far edge of the range as much as at a date somebody picked.
        Assert.Equal("0001-01-01", Runtime.Written(DateOnly.MinValue));
        Assert.Equal("9999-12-31", Runtime.Written(DateOnly.MaxValue));

        // And `Written` takes an `object?`, which promises text for whatever it is handed rather
        // than only for the shapes it enumerates. A type this package has never heard of travels as
        // what that type says it is, and never as nothing at all — a query value that came back
        // null would be a request nobody could send.
        Assert.Equal("a value of its own", Runtime.Written(new Unlisted()));
        Assert.NotNull(Runtime.Written(new object()));
    }

    [Fact]
    public void ARequestThisPackageCouldNotIssueIsRefusedRatherThanBuilt()
    {
        // Neither bound is reachable through a generated method: the emitter writes one call per
        // operation and the document declares nothing near either ceiling. They are what keeps a
        // caller reaching for `Runtime` directly, or a document that grew out of shape, from
        // building a request nobody meant.
        List<(string Name, object? Value)> many =
            [.. Enumerable.Range(0, Runtime.MaxPathParameters + 1).Select(at => ($"p{at}", (object?)at))];

        Assert.Throws<ArgumentOutOfRangeException>(() => Runtime.Path("/api/v1/things/{p0}", many));
        Assert.Throws<ArgumentOutOfRangeException>(() => Runtime.Query(
            [],
            [.. Enumerable.Range(0, Runtime.MaxQueryParameters + 1).Select(at => ($"q{at}", (object?)at))]));
    }

    [Fact]
    public void AnAnswerAboveWhatThisPackageReadsIsRefusedBeforeItIsParsed()
    {
        byte[] oversized = new byte[Runtime.MaxPayloadBytes + 1];

        DecodeException refused = Assert.Throws<DecodeException>(() => Runtime.Read<Problem>(oversized));

        Assert.Contains("above the", refused.Message, StringComparison.Ordinal);
    }

    [Fact]
    public void AnAnswerThatIsNotJsonAtAllIsRefusedAsOneTheApiDoesNotDeclare()
    {
        byte[] answered = Encoding.UTF8.GetBytes("<html>a proxy answered this</html>");

        DecodeException refused = Assert.Throws<DecodeException>(() => Runtime.Read<Problem>(answered));

        Assert.NotNull(refused.InnerException);
        Assert.Null(Runtime.ReadOrNothing<Problem>(answered));
    }

    [Fact]
    public void AnAnswerCarryingTheWordNothingIsNotAValueTheApiDeclares()
    {
        // `null` is a document, and reading it succeeds; what it carries is not a value, and a
        // caller handed one would meet the failure a member later rather than here.
        DecodeException refused = Assert.Throws<DecodeException>(
            () => Runtime.Read<Problem>(Encoding.UTF8.GetBytes("null")));

        Assert.Contains("carries no Problem", refused.Message, StringComparison.Ordinal);
    }

    [Fact]
    public void ABodyIsQuotedBackAtAFixedBudgetRatherThanWhole()
    {
        // What goes into a failure message is written by a server this package does not control, so
        // it is cut rather than echoed into whatever the caller logs.
        string quoted = Runtime.Preview(Encoding.UTF8.GetBytes(new string('x', Runtime.MaxPreviewBytes + 1)));

        Assert.Equal(Runtime.MaxPreviewBytes + 1, quoted.Length);
        Assert.EndsWith("…", quoted, StringComparison.Ordinal);
    }

    [Fact]
    public void AProblemThisClientHasNeverHeardOfIsStillReadAsAFailure()
    {
        byte[] answered = Encoding.UTF8.GetBytes(
            new JsonObject { ["id"] = "AProblemThisClientHasNeverHeardOf", ["status"] = 400 }.ToJsonString());

        ProblemException raised = Assert.Throws<ProblemException>(
            () => Problems.RaiseForStatus(400, answered));

        Assert.Equal(400, raised.Status);
    }

    [Fact]
    public void ABodyThatIsNotAProblemIsStillReadAsAFailure()
    {
        ProblemException raised = Assert.Throws<ProblemException>(
            () => Problems.RaiseForStatus(502, Encoding.UTF8.GetBytes("<html>a proxy answered</html>")));

        Assert.Equal(502, raised.Status);
        Assert.Null(raised.Problem);
    }

    [Fact]
    public void ASuccessRaisesNothing() => Problems.RaiseForStatus(204, []);

    [Fact]
    public void EveryGeneratedValueReadsBackWhatItWrote()
    {
        // Every value the generator wrote, discovered from what it wrote and filled out of what it
        // declares: a schema the document adds joins this case the moment the generated file carries
        // it, and one whose reader and writer disagree about a member fails here rather than at the
        // first answer that carries it.
        int exercised = 0;
        foreach (Type declared in Values)
        {
            JsonNode filled = Document(declared, MaxNesting);
            JsonNode written = Rewritten(declared, filled);

            // Every member the document names travels back under the name it arrived under, and
            // nothing else does.
            Assert.Equal(Names(filled), Names(written));

            // Reading what was written and writing it again lands on the same document, which is
            // what says the reader and the writer agree about every member rather than about the
            // ones a hand-written case happened to name.
            Assert.Equal(Canonical(written), Canonical(Rewritten(declared, written)));
            exercised++;
        }

        Assert.True(exercised > 0, "no value the generator wrote could be filled, so this guard checked nothing");
    }

    [Fact]
    public void EveryMemberTheDocumentDoesNotRequireIsAbsentRatherThanWrittenBackAsNothing()
    {
        // Every value, and every member of it, one member at a time: the document arrives without it
        // and what happens next has to be one of two things. Either the member is required and the
        // read stops, or it is not and the value reads — in which case writing it back leaves the
        // member out rather than answering it as nothing, which is a difference the API can see.
        int required = 0;
        int optional = 0;

        foreach (Type declared in Values)
        {
            JsonObject whole = Document(declared, MaxNesting).AsObject();
            foreach (string name in whole.Select(member => member.Key).ToList())
            {
                JsonObject without = Document(declared, MaxNesting).AsObject();
                without.Remove(name);

                JsonNode? back;
                try
                {
                    back = Rewritten(declared, without);
                }
                catch (JsonException)
                {
                    required++;
                    continue;
                }

                Assert.False(
                    back!.AsObject().ContainsKey(name),
                    $"{declared.Name}.{name} was answered none of and written back as nothing");
                Assert.Equal(Names(without), Names(back));
                optional++;
            }
        }

        Assert.True(required > 0, "no member the document requires was found, so this guard checked nothing");
        Assert.True(optional > 0, "no member the document leaves out was found, so this guard checked nothing");
    }

    [Theory]
    [InlineData(Surface.Blocking)]
    [InlineData(Surface.Awaiting)]
    public async Task EveryOperationTheApiDeclaresIsIssuedByTheMethodWrittenForIt(Surface surface)
    {
        // What an SDK is for: the API declares an operation, and the generated half issues exactly
        // that request for it and reads back exactly what the answer carried. Both sides are
        // discovered — the operations out of the API's own description, the methods out of what the
        // generator wrote — so an operation the document adds and the generator misses fails here
        // rather than at whichever caller reached for it first.
        IReadOnlyDictionary<string, DeclaredOperation> declared = Declarations();
        SortedSet<string> issued = new(StringComparer.Ordinal);

        foreach (Type group in Groups(surface))
        {
            foreach (MethodInfo operation in Methods(group))
            {
                string named = $"{group.Name}.{operation.Name}";
                Type? answers = Answers(operation);
                JsonNode? answered = answers is null ? null : Document(answers, MaxNesting);
                object?[] arguments = ArgumentsOf(operation);

                Restarted();
                Api.WillAnswer(new ScriptedResponse(200, answered), new ScriptedResponse(200, answered?.DeepClone()));
                using Hook0Client client = Client();
                object driven = Activator.CreateInstance(group, client.Transport)
                    ?? throw new InvalidOperationException($"`{group.Name}` is not built on a transport");

                object? read = await CalledAsync(operation, driven, arguments);
                if (answers is not null)
                {
                    // Held against the same answer read and written back rather than against the
                    // document as it was built, so that what is asserted is the value that arrived
                    // rather than the spelling a serialiser happens to write a moment in.
                    Assert.Equal(
                        Canonical(Rewritten(answers, answered!)),
                        Canonical(JsonNode.Parse(JsonSerializer.Serialize(read, answers, Runtime.WritingOptions))!));
                }

                ReceivedRequest sent = Api.Received[0];
                DeclaredOperation? matched = Matching(declared, sent.Verb, sent.Target.Split('?')[0]);
                Assert.True(matched is not null, $"{named} issued `{sent.Target}`, which the API declares nothing at");
                issued.Add(matched!.Named);

                foreach (object? argument in arguments)
                {
                    if (argument is string carried)
                    {
                        Assert.Contains(carried, sent.Target, StringComparison.Ordinal);
                    }
                    else if (argument is not null and not CancellationToken)
                    {
                        Assert.Equal(
                            Canonical(JsonNode.Parse(JsonSerializer.Serialize(argument, Runtime.WritingOptions))!),
                            Canonical(sent.Json()));
                    }
                }

                Assert.Equal(matched.Query, Assembled(sent));

                if (matched.Query.SetEquals(matched.Required))
                {
                    continue;
                }

                // The same operation again, with everything the API does not ask for left out: a
                // parameter a caller sends none of has to be absent from the query rather than sent
                // as an empty one, which is a difference the API can see.
                await CalledAsync(operation, driven, WithoutTheOptional(arguments, sent, matched));

                Assert.Equal(matched.Required, Assembled(Api.Received[1]));
            }
        }

        Assert.Equal(declared.Keys.OrderBy(name => name, StringComparer.Ordinal), issued);
    }

    [Fact]
    public void AValueTravelsUnderTheNamesTheDocumentDeclares()
    {
        Application application = new()
        {
            ApplicationId = Guid.Parse("00000000-0000-4000-8000-000000000001"),
            Name = "an application",
            OrganizationId = Guid.Parse("00000000-0000-4000-8000-000000000002"),
        };

        JsonNode written = JsonNode.Parse(
            JsonSerializer.Serialize(application, Runtime.WritingOptions))!;

        Assert.Equal("an application", written["name"]!.GetValue<string>());
        Assert.NotNull(written["application_id"]);
        Assert.NotNull(written["organization_id"]);
        Assert.Equal(application, JsonSerializer.Deserialize<Application>(written.ToJsonString(), Runtime.ReadingOptions));
    }

    [Fact]
    public void AMemberTheDocumentRequiresIsRefusedWhenItIsAbsent()
    {
        Assert.ThrowsAny<JsonException>(() =>
            JsonSerializer.Deserialize<Application>("{\"name\": \"an application\"}", Runtime.ReadingOptions));
    }

    [Theory]
    [InlineData(Surface.Blocking)]
    [InlineData(Surface.Awaiting)]
    public async Task AGeneratedOperationIssuesTheRequestTheDocumentDescribes(Surface surface)
    {
        Api.WillAnswer(new ScriptedResponse(
            200,
            new JsonArray(new JsonObject
            {
                ["application_id"] = "00000000-0000-4000-8000-000000000001",
                ["name"] = "an application",
                ["organization_id"] = "00000000-0000-4000-8000-000000000002",
            })));
        using Hook0Client client = Client();

        IReadOnlyList<Application> listed = surface == Surface.Blocking
            ? await Task.Run(() => new ApplicationsApi(client.Transport).List("an-organization"))
            : await new ApplicationsAsyncApi(client.Transport).ListAsync("an-organization");

        Application only = Assert.Single(listed);
        Assert.Equal("an application", only.Name);

        ReceivedRequest issued = Assert.Single(Api.Received);
        Assert.Equal("GET", issued.Verb);
        Assert.Equal("/api/v1/applications/?organization_id=an-organization", issued.Target);
        Assert.Empty(issued.Body);
    }

    [Theory]
    [InlineData(Surface.Blocking)]
    [InlineData(Surface.Awaiting)]
    public async Task AGeneratedOperationFillsInThePathItIsGiven(Surface surface)
    {
        Api.WillAnswer(new ScriptedResponse(204, null));
        using Hook0Client client = Client();

        if (surface == Surface.Blocking)
        {
            await Task.Run(() => new ApplicationsApi(client.Transport).Delete("an application/../.."));
        }
        else
        {
            await new ApplicationsAsyncApi(client.Transport).DeleteAsync("an application/../..");
        }

        ReceivedRequest issued = Assert.Single(Api.Received);
        Assert.Equal("DELETE", issued.Verb);

        // Every character the segment carries that could name another one travels encoded.
        Assert.Equal(
            "/api/v1/applications/an%20application%2F..%2F..",
            issued.Target);
    }

    [Theory]
    [InlineData(Surface.Blocking)]
    [InlineData(Surface.Awaiting)]
    public async Task AGeneratedOperationRaisesWhatTheApiReported(Surface surface)
    {
        Api.WillAnswer(new ScriptedResponse(404, FakeApi.Problem("NotFound", 404)));
        using Hook0Client client = Client();

        NotFoundException raised = surface == Surface.Blocking
            ? await Assert.ThrowsAsync<NotFoundException>(
                () => Task.Run(() => new ApplicationsApi(client.Transport).Get("an-application")))
            : await Assert.ThrowsAsync<NotFoundException>(
                () => new ApplicationsAsyncApi(client.Transport).GetAsync("an-application"));

        Assert.Equal(404, raised.Status);
    }

    /// <summary>One operation the API declares for an SDK: where it lands, and what its query names.</summary>
    /// <param name="Verb">The method it is issued under.</param>
    /// <param name="Path">Where it lands, with a placeholder per member of the path it names.</param>
    /// <param name="Query">Every parameter it takes in its query string.</param>
    /// <param name="Required">The ones of those a caller cannot leave out.</param>
    private sealed record DeclaredOperation(
        string Verb,
        string Path,
        SortedSet<string> Query,
        SortedSet<string> Required)
    {
        /// <summary>How this operation is named where one is held against a request that arrived.</summary>
        public string Named => $"{Verb} {Path}";
    }

    /// <summary>
    /// Every operation the API declares for an SDK, by the verb and the path it declares it under.
    /// </summary>
    /// <remarks>
    /// Only the ones the document marks public: those are what the generator writes a method for,
    /// and holding the generated half against the rest would be holding it against operations it
    /// was never asked to carry.
    /// </remarks>
    private static IReadOnlyDictionary<string, DeclaredOperation> Declarations()
    {
        Dictionary<string, DeclaredOperation> found = new(StringComparer.Ordinal);
        foreach (KeyValuePair<string, JsonNode?> path in Corpus.Description()["paths"]!.AsObject())
        {
            foreach (KeyValuePair<string, JsonNode?> operation in path.Value!.AsObject())
            {
                JsonArray tags = operation.Value!["tags"]?.AsArray() ?? [];
                if (!tags.Any(tag => tag!.GetValue<string>() == PublicTag))
                {
                    continue;
                }

                DeclaredOperation declared = new(
                    operation.Key.ToUpperInvariant(),
                    path.Key,
                    QueryOf(operation.Value!, onlyRequired: false),
                    QueryOf(operation.Value!, onlyRequired: true));
                found[declared.Named] = declared;
            }
        }

        Assert.NotEmpty(found);
        return found;
    }

    /// <summary>What one operation names in its query string, either all of it or only what is asked for.</summary>
    private static SortedSet<string> QueryOf(JsonNode operation, bool onlyRequired)
    {
        SortedSet<string> named = new(StringComparer.Ordinal);
        foreach (JsonNode? parameter in operation["parameters"]?.AsArray() ?? [])
        {
            bool required = parameter!["required"]?.GetValue<bool>() == true;
            if (parameter["in"]?.GetValue<string>() != "query" || (onlyRequired && !required))
            {
                continue;
            }

            named.Add(parameter["name"]!.GetValue<string>());
        }

        return named;
    }

    /// <summary>The operation groups of one flavour, in a settled order.</summary>
    private static IEnumerable<Type> Groups(Surface surface) => Declared.Where(declared =>
        declared.Name.EndsWith("AsyncApi", StringComparison.Ordinal) == (surface == Surface.Awaiting)
        && declared.Name.EndsWith("Api", StringComparison.Ordinal));

    /// <summary>Every value the generator wrote, which is what carries members the document names.</summary>
    private static IEnumerable<Type> Values =>
        Declared.Where(declared => declared.IsClass && MembersOf(declared).Any());

    /// <summary>What one value declares, under the names the document declares them with.</summary>
    private static IEnumerable<PropertyInfo> MembersOf(Type declared) =>
        declared.GetProperties(BindingFlags.Public | BindingFlags.Instance)
            .Where(member => member.GetCustomAttribute<System.Text.Json.Serialization.JsonPropertyNameAttribute>()
                is not null)
            .OrderBy(member => member.Name, StringComparer.Ordinal);

    /// <summary>
    /// A document of that shape, with every member the document names carrying something.
    /// </summary>
    /// <remarks>
    /// Built out of what the declaration carries rather than out of a document written here, so a
    /// member the API adds is filled in the moment the generated file carries it — and one this
    /// suite cannot build is a failure saying so rather than a case that quietly stops covering it.
    /// </remarks>
    private static JsonNode Document(Type declared, int depth)
    {
        if (depth <= 0)
        {
            throw new InvalidOperationException($"`{declared.Name}` nests deeper than the {MaxNesting} built");
        }

        Type held = Nullable.GetUnderlyingType(declared) ?? declared;
        if (held == typeof(string))
        {
            return "what the document names";
        }

        if (held == typeof(Guid))
        {
            return "3f2504e0-4f89-41d3-9a0c-0305e82c3310";
        }

        if (held == typeof(int) || held == typeof(long) || held == typeof(short))
        {
            return 7;
        }

        if (held == typeof(double) || held == typeof(float) || held == typeof(decimal))
        {
            return 1.5;
        }

        if (held == typeof(bool))
        {
            return true;
        }

        if (held == typeof(DateTimeOffset) || held == typeof(DateTime))
        {
            return "2026-01-02T03:04:05Z";
        }

        if (held == typeof(DateOnly))
        {
            return "2026-01-02";
        }

        if (held == typeof(JsonNode) || held == typeof(JsonElement) || held == typeof(object))
        {
            // A member the document leaves undescribed, which therefore travels as whatever arrived.
            return new JsonObject { ["a member the document does not describe"] = "what it carried" };
        }

        if (held.IsGenericType)
        {
            Type[] arguments = held.GetGenericArguments();
            if (arguments.Length == 1)
            {
                return new JsonArray(Document(arguments[0], depth - 1));
            }

            if (arguments.Length == 2)
            {
                return new JsonObject
                {
                    ["a key the document leaves open"] = Document(arguments[1], depth - 1),
                };
            }
        }

        JsonObject written = [];
        foreach (PropertyInfo member in MembersOf(held))
        {
            written[Named(member)] = Document(member.PropertyType, depth - 1);
        }

        if (written.Count == 0)
        {
            throw new InvalidOperationException($"`{held.Name}` is a shape this suite cannot build");
        }

        return written;
    }

    /// <summary>The name one member travels under.</summary>
    private static string Named(PropertyInfo member) =>
        member.GetCustomAttribute<System.Text.Json.Serialization.JsonPropertyNameAttribute>()!.Name;

    /// <summary>That document read as the value the API declares, and written straight back out.</summary>
    private static JsonNode Rewritten(Type declared, JsonNode document) =>
        JsonNode.Parse(
            JsonSerializer.Serialize(
                JsonSerializer.Deserialize(document.ToJsonString(), declared, Runtime.ReadingOptions),
                declared,
                Runtime.WritingOptions))!;

    /// <summary>The names one document carries, in a settled order.</summary>
    private static IEnumerable<string> Names(JsonNode document) =>
        document.AsObject().Select(member => member.Key).OrderBy(name => name, StringComparer.Ordinal);

    /// <summary>One document as a text two of them can be compared by, whatever order they were built in.</summary>
    private static string Canonical(JsonNode document) => document switch
    {
        JsonObject members => "{"
            + string.Join(",", members.OrderBy(member => member.Key, StringComparer.Ordinal)
                .Select(member => $"{JsonSerializer.Serialize(member.Key)}:{Canonical(member.Value!)}"))
            + "}",
        JsonArray items => "[" + string.Join(",", items.Select(item => Canonical(item!))) + "]",
        _ => document.ToJsonString(),
    };

    /// <summary>What one operation is declared to answer, or nothing where it answers none.</summary>
    private static Type? Answers(MethodInfo operation)
    {
        Type answered = operation.ReturnType;
        if (answered == typeof(void) || answered == typeof(Task))
        {
            return null;
        }

        return answered.IsGenericType && answered.GetGenericTypeDefinition() == typeof(Task<>)
            ? answered.GetGenericArguments()[0]
            : answered;
    }

    /// <summary>What one operation is called with: a distinct value per name it takes, and one per body.</summary>
    private static object?[] ArgumentsOf(MethodInfo operation)
    {
        ParameterInfo[] shape = operation.GetParameters();
        object?[] arguments = new object?[shape.Length];
        for (int at = 0; at < shape.Length; at++)
        {
            arguments[at] = shape[at].ParameterType switch
            {
                Type declared when declared == typeof(string) => $"what-this-case-passed-{at}",
                Type declared when declared == typeof(CancellationToken) => CancellationToken.None,
                Type declared => JsonSerializer.Deserialize(
                    Document(declared, MaxNesting).ToJsonString(),
                    declared,
                    Runtime.ReadingOptions),
            };
        }

        return arguments;
    }

    /// <summary>What one operation answered, whichever flavour it was called through.</summary>
    private static async Task<object?> CalledAsync(MethodInfo operation, object group, object?[] arguments)
    {
        object? outcome;
        try
        {
            outcome = operation.Invoke(group, arguments);
        }
        catch (TargetInvocationException raised)
        {
            throw raised.InnerException ?? raised;
        }

        if (outcome is not Task waiting)
        {
            return outcome;
        }

        await waiting.ConfigureAwait(false);
        return waiting.GetType().IsGenericType ? waiting.GetType().GetProperty("Result")!.GetValue(waiting) : null;
    }

    /// <summary>The one operation the API declares under that verb and that path, or nothing at all.</summary>
    private static DeclaredOperation? Matching(
        IReadOnlyDictionary<string, DeclaredOperation> declared,
        string verb,
        string path)
    {
        DeclaredOperation? found = null;
        foreach (DeclaredOperation operation in declared.Values)
        {
            if (operation.Verb != verb || !Fills(operation.Path, path))
            {
                continue;
            }

            Assert.True(found is null, $"`{verb} {path}` reads as both `{found?.Named}` and `{operation.Named}`");
            found = operation;
        }

        return found;
    }

    /// <summary>Whether that path is that template with every placeholder of it filled in.</summary>
    private static bool Fills(string template, string path)
    {
        string[] declared = template.Split('/');
        string[] issued = path.Split('/');
        if (declared.Length != issued.Length)
        {
            return false;
        }

        for (int at = 0; at < declared.Length; at++)
        {
            bool placeholder = declared[at].StartsWith('{') && declared[at].EndsWith('}');
            if (placeholder ? issued[at].Length == 0 : declared[at] != issued[at])
            {
                return false;
            }
        }

        return true;
    }

    /// <summary>The parameters one request carried in its query string, by name.</summary>
    private static SortedSet<string> Assembled(ReceivedRequest sent)
    {
        SortedSet<string> named = new(StringComparer.Ordinal);
        string[] target = sent.Target.Split('?');
        if (target.Length < 2 || target[1].Length == 0)
        {
            return named;
        }

        foreach (string parameter in target[1].Split('&'))
        {
            named.Add(Uri.UnescapeDataString(parameter.Split('=')[0]));
        }

        return named;
    }

    /// <summary>The same arguments, less the ones that landed under a parameter the API does not ask for.</summary>
    private static object?[] WithoutTheOptional(
        object?[] arguments,
        ReceivedRequest sent,
        DeclaredOperation matched)
    {
        object?[] left = [.. arguments];
        for (int at = 0; at < left.Length; at++)
        {
            if (left[at] is not string carried)
            {
                continue;
            }

            if (matched.Query.Any(parameter =>
                !matched.Required.Contains(parameter)
                && sent.Target.Contains($"{parameter}={carried}", StringComparison.Ordinal)))
            {
                left[at] = null;
            }
        }

        return left;
    }

    /// <summary>
    /// What one declaration actually carries, which is not everything reflection reports: a record
    /// answers to `ToString`, `Equals` and `GetHashCode` because the compiler writes them, and a
    /// member the emitter had spelled that way would not have compiled at all.
    /// </summary>
    private static IEnumerable<MemberInfo> Written(Type declared) =>
        declared.GetMembers(BindingFlags.Public | BindingFlags.Instance | BindingFlags.Static
                | BindingFlags.DeclaredOnly)
            .Where(member => member is not ConstructorInfo)
            .Where(member => member.GetCustomAttribute<CompilerGeneratedAttribute>() is null);

    /// <summary>What one operation group declares, by name.</summary>
    private static IReadOnlySet<string> Operations(Type group) =>
        new HashSet<string>(Methods(group).Select(method => method.Name), StringComparer.Ordinal);

    private static IEnumerable<MethodInfo> Methods(Type group) =>
        group.GetMethods(BindingFlags.Public | BindingFlags.Instance | BindingFlags.DeclaredOnly)
            .Where(method => !method.IsSpecialName);

    /// <summary>A Task-returning method under the name its blocking twin carries.</summary>
    private static string Shortened(string name) =>
        name.EndsWith(AsyncSuffix, StringComparison.Ordinal)
            ? name[..^AsyncSuffix.Length]
            : name;
    /// <summary>A type this package has no arm for, which is every type a caller writes itself.</summary>
    private sealed class Unlisted
    {
        /// <inheritdoc />
        public override string ToString() => "a value of its own";
    }
}
