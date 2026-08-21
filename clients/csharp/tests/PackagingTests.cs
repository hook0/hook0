// What installing this package is allowed to drag in, which is nothing.
//
// The package reaches the network, verifies signatures and decodes what the API answers with the
// framework alone. That sentence is worth exactly as much as the guard behind it, so it is a case
// rather than a line in a pipeline: a `PackageReference` appearing in the project file fails here,
// wherever the suite runs.

using System;
using System.Collections.Generic;
using System.IO;
using System.Linq;
using System.Xml.Linq;
using Xunit;

namespace Hook0.Tests;

/// <summary>What the project file says the package is.</summary>
public sealed class PackagingTests
{
    /// <summary>Largest project file read back.</summary>
    private const int MaxProjectBytes = 64 * 1024;

    /// <summary>How far up the tree the project is looked for before the search gives up.</summary>
    private const int MaxLevels = 10;

    private static readonly XDocument Project = Read();

    [Fact]
    public void ThePackageDeclaresNoRuntimeDependency()
    {
        List<string> declared =
        [
            .. Project.Descendants("PackageReference")
                .Select(reference => reference.Attribute("Include")?.Value ?? "an unnamed reference"),
        ];

        Assert.True(
            declared.Count == 0,
            "the package has grown a runtime dependency; it is meant to reach for the framework " +
            $"alone: {string.Join(", ", declared)}");
    }

    [Fact]
    public void ThePackageIsPublishedUnderTheIdentityTheRestOfTheSdksShare()
    {
        Assert.Equal("Hook0.Client", Property("PackageId"));
        Assert.Equal("MIT", Property("PackageLicenseExpression"));
        Assert.False(string.IsNullOrWhiteSpace(Property("Version")));
        Assert.False(string.IsNullOrWhiteSpace(Property("Description")));
    }

    [Fact]
    public void ThePackageIsBuiltUnderTheDisciplineItsSourceIsWrittenFor()
    {
        // The generator emits already-formatted, already-documented source. These three are what
        // turn that into something the build checks rather than something a reader hopes for.
        Assert.Equal("true", Property("TreatWarningsAsErrors"));
        Assert.Equal("true", Property("GenerateDocumentationFile"));
        Assert.Equal("enable", Property("Nullable"));
    }

    [Fact]
    public void ThePackageShipsBothHalvesOfWhatItIs()
    {
        DirectoryInfo source = new(Path.GetDirectoryName(Located())!);

        Assert.True(File.Exists(Path.Combine(source.FullName, "Hook0", "Signature.cs")));
        Assert.True(File.Exists(Path.Combine(source.FullName, "Hook0", "Client.cs")));
        Assert.True(File.Exists(Path.Combine(source.FullName, "Hook0", "Generated", "Models.cs")));
        Assert.True(File.Exists(Path.Combine(source.Parent!.FullName, "README.md")));
    }

    private static string Property(string name) =>
        Project.Descendants(name).FirstOrDefault()?.Value.Trim() ?? string.Empty;

    private static XDocument Read()
    {
        string path = Located();
        long size = new FileInfo(path).Length;
        if (size > MaxProjectBytes)
        {
            throw new InvalidOperationException(
                $"{path} is {size} bytes long, above the {MaxProjectBytes} read back");
        }

        return XDocument.Parse(File.ReadAllText(path));
    }

    private static string Located()
    {
        DirectoryInfo? walked = new(AppContext.BaseDirectory);
        for (int level = 0; level < MaxLevels && walked is not null; level++)
        {
            string candidate = Path.Combine(walked.FullName, "src", "Hook0.Client.csproj");
            if (File.Exists(candidate))
            {
                return candidate;
            }

            walked = walked.Parent;
        }

        throw new FileNotFoundException(
            $"no `src/Hook0.Client.csproj` sits within {MaxLevels} levels of {AppContext.BaseDirectory}");
    }
}
