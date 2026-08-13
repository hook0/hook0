# Contributing

Heya! Welcome to Hook0, and thank you for taking the time to contribute back to Open Source Software! ❤️ We want
everybody to be able to contribute to Hook0, no matter your background or expertise. In order to facilitate that,
we've put together a couple tips and tricks below. Our team truly appreciates every single contributor, community
member, GitHub star, pull-request, bug report, and feature request. Keeping Hook0 completely free and open-source is
our way of saying: **Thank you!**

> We're here to help!
>
> If you have _any_ questions along your contributor journey, please feel free to come chat with us on
> [our Discord](https://www.hook0.com/community).

```sh
# Clone the repository
git clone https://gitlab.com/hook0/hook0.git

# Navigate to the project directory
cd hook0

# Install dependencies
cargo build

# Run the application
cargo run
```

## Working on the API

Two rules apply to every endpoint of the API, and tests enforce both:

- An endpoint reaches the generated SDKs only if its `#[api_v2_operation(...)]` carries the `public`
  tag. That tag publishes it into the twelve SDKs we generate and freezes its shape: renaming it
  or changing its responses afterwards breaks code we do not control. Leave it off unless the
  endpoint is part of the product our users integrate against. Tagged operations name themselves
  `entity.verb`, with `list`, `get`, `create`, `update` and `delete` as the expected verbs.
- The OpenAPI document is committed in `api/openapi.snapshot.json`, and a test compares it with the
  one the application serves. Touching a handler makes that test fail; adopt the change with
  `UPDATE_OPENAPI_SNAPSHOT=1 cargo test -p hook0-api openapi_snapshot` and commit the rewritten
  snapshot.

[`api/README.md`](./api/README.md) explains when the tag is warranted and what to read in the
snapshot report.

## Code of Conduct

**The Hook0 [Code of Conduct](CODE_OF_CONDUCT.md) is one of the ways
we put our values into practice. We expect all of our staff, contractors and contributors to know and follow this
code.**

**Our contributors and maintainers work extremely hard to build Hook0 as premium open-source software. Please be
respectful of those efforts throughout our ecosystem. Trolling, harassing, insulting, or other unacceptable behavior by
participants will not be tolerated.**

In the interest of fostering an open and welcoming environment, we as contributors and maintainers pledge to making
participation in our project and community a harassment-free experience for everyone, regardless of age, body size,
disability, ethnicity, sex characteristics, gender identity and expression, level of experience, education,
socio-economic status, nationality, personal appearance, race, religion, or sexual identity and orientation.

Examples of behavior that contributes to creating a positive environment include:

- Using welcoming and inclusive language
- Being respectful of differing viewpoints and experiences
- Gracefully accepting constructive criticism
- Focusing on what is best for the community
- Showing empathy towards other community members

**Before continuing, please take a moment to read our full
[Code of Conduct](CODE_OF_CONDUCT.md).**

## Ways to Contribute

### Reporting Bugs

If you happen to run into a bug, please post an Issue on our main GitHub Issue board:
https://gitlab.com/hook0/hook0/-/issues

Please be as explicit and detailed as you can in the bug report. The more information available, the easier it is for
other contributors to help you find the solution or fix. Consider adding a schema snapshot file, or a database dump.

### Leaving Feedback

If you have a great idea for an improvement of the platform, or any other feedback, please make sure to open a new
Issue: https://gitlab.com/hook0/hook0/-/issues

### Document the Project

The [Hook0 Docs](https://documentation.hook0.com/) are living documents that can always be improved on. Notice any
parts of the docs in dire need of some tender love and care? Feel free to Suggest Edits (link at the top right-hand
corner on every documentation page)!

### Helping Others

The Hook0 community is growing quickly, which also means there's more and more people that have questions. Helping
out your fellow developers by answering questions on [Discord](https://www.hook0.com/community) or
[Gitlab issues](https://gitlab.com/hook0/hook0/-/issues/?sort=milestone_due_desc&state=opened&label_name%5B%5D=Community%3A%3Acontribution&first_page_size=20)
is a great way to help the
project.

### Pull Requests

#### Bug Fixes

We treat Issues on the main repo as actionable items we want to get done. This also means that we welcome PRs for any
Issue that has been labeled either "Bug", "Improvement", or "New Feature". Labeled issues are bugs or new features that
have been triaged, accepted, and are ready to be implemented.

#### Implementing Features

With the continuous growth of Hook0, more and more people are relying on Hook0 for (critical) data workloads in
various use cases. This means we need to be careful with any changes that might affect the stability, security,
performance, or scalability of Hook0. For this reason, it's important that any new feature is properly thought
through and discussed before being implemented.

Before you start writing code to implement your new feature idea, please read through and understand our triaging
process for new features before diving in. While we encourage and appreciate every code contribution, please understand
that we can't merge every suggested code change.

##### Triaging Process

Feature Request Discussions that are deemed ready to be implemented with the discussed implementation details are marked
"Accepted" and converted into an Issue, at which point the feature is ready to be implemented.

New feature ideas reported directly to issues might be converted into a Discussion for further triaging at
[the core team](https://gitlab.com/hook0/hook0/-/project_members)'s discretion first. This is often due to a lack of
detail, or
lack of proven interest.

Each Pull Request that comes in is required to resolve [an open Issue](https://gitlab.com/hook0/hook0/-/issues) that
is labeled "Bug", "Improvement", or "New Feature". This ensures that any code change made implements a known actionable
item, be it a feature or otherwise.

#### Rules Implemented Twice

A few rules exist in both the API and the frontend, so a form can refuse a value without a round trip. When that
happens, the two implementations are pinned to a single fixture rather than trusted to stay in step by hand.

The password policy is the current example: `password-policy-vectors.json`, at the root of the repository, is read by
the Rust suite (`api/src/password.rs`) and by the frontend suite (`frontend/src/utils/passwordPolicy.test.ts`,
`frontend/src/utils/passwordProblem.test.ts`). It fixes the verdict for a list of passwords, the maximum length, and
the set of API error ids that mean "the password was refused". Changing a rule on one side without the other fails
both suites, which is the point: a form that accepts what the API refuses wastes the user's time, and a form that
refuses what the API accepts is a bug nobody sees.

If you add a rejection reason, or move a bound, put it in the fixture in the same change.

### Reporting Security Vulnerabilities

If you believe you have discovered a security issue within a Hook0 product or service, please reach out to us
directly over email: [security@hook0.com](mailto:security@hook0.com). We will then open a
[Gitlab issue](https://gitlab.com/hook0/hook0/-/issues) for tracking the fix.

We value the members of the independent security research community who find security vulnerabilities and work with our
team so that proper fixes can be issued to users. Our policy is to credit all researchers in the fix's release notes. In
order to receive credit, security researchers must follow responsible disclosure practices, including:

- They do not publish the vulnerability prior to the Hook0 team releasing a fix for it
- They do not divulge exact details of the issue, e.g., through exploits or proof-of-concepts
