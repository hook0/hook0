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
}
