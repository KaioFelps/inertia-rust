# Introduction

**Inertia Rust** is a Rust adapter for Inertia.js.

It's not its aim to work around a single specific framework. Instead, it's built out
of providers, which are responsible for implementing the crate's trats for a given framework.

Currently, only **actix-web** has a provider. As it's the first --- and the only --- existing provider,
every code block will be considering it as the underlying framework. Note, however, that the code shouldn't
be very different depending on the opted provider.

We will cover only how to make things that are specific to Inertia Rust. For the front-end, refer to
[Inertia's oficial documentation](https://inertiajs.com/).

> Note: This documentation describes how to use `inertia-rust v2.x`, which is compatible with Inertia.js
> v2.0.0. Also, it has notable differences comparing to the former v0.1.0.
>
> Indeed, we went from v0.1.0 straight to v2.0.0. The reason is we wanted to let it as clear as possible
> that this is the correct version of the crate to work with Inertia.js v2.

## Inertia Rust v0.1.0
If you want to use Inertia.js 1, you must refer to a previous version of Inertia Rust. Unfortunately, there
is no such a documentation about it. We hope you can make it using the following files:

<table><tbody>
<thead>
<tr>
    <th> File </th>
    <th> Content </th>
</tr>
</thead>
<tr>
<td>

[v0.1.0 README.md]
        
</td>
<td>

* How to set up inertia-rust with vite-rust;
* How to use and/or create template resolvers;
* How to enable SSR;
* Render pages;
* Enable Inertia Middleware and share props.
        
</td>
</tr>
<tr>
<td>

[v0.1.0 REQUIREMENTS.md]

</td>
<td>

Not actually a requirements list for you to use Inertia, but of an lib to be a useful Inertia Adapter:

* Create valid Inertia responses;
* Create valid Inertia routes;
* Generating error props;
* Redirects.

</td>
</tr>
<tr>
<td>

[CHANGELOG.md]

</td>
<td>

A changelog of every change that has occurred between v0.1.0 and v2.0.0, including the good-to-know
breaking changes. Note that developer experience --- DX --- has been drastically enhanced from the former
to the latter version.

</td>
</tr>
<tbody></table>

[v0.1.0 README.md]: https://github.com/KaioFelps/inertia-rust/blob/v1/README.md
[v0.1.0 REQUIREMENTS.md]: https://github.com/KaioFelps/inertia-rust/blob/v1/REQUIREMENTS.md
[CHANGELOG.md]: https://github.com/KaioFelps/inertia-rust/blob/v2/CHANGELOG.md