# Routes and Responses

Creating routes is totally up to the framework you're using for building your application, of course. However,
we provide some facilities. For instance, you can render a simple page both ways:

```rust
use inertia_rust::{Inertia, InertiaFacade, InertiaService};
use actix_web::{app, get, Responder, HttpRequest};

// this
App::new().inertia_route("/", "Index");

// is the same as this
#[get("/")]
async fn index(req: HttpRequest) -> impl Responder {
    Inertia::render(&req, "Index".into()).await
}

App::new().service(index);
```

## Redirects
If you want to redirect to another page, it's enough to return your framework Redirect response. However, when
redirecting to an external site, make sure to use `Inertia::location` method. It'll defer the redirect to the
browser:

```rust
use inertia_rust::{Inertia, InertiaFacade};
use actix_web::{get, Responder, HttpRequest};

#[get("/")]
async fn index(req: HttpRequest) -> impl Responder {
    Inertia::location(&req, "https://inertiajs.com")
}
```

To redirect to the previous page (a.k.a **redirect back**), use the `back` facade method:

```rust
use inertia_rust::{Inertia, InertiaFacade};
use actix_web::{get, Responder, HttpRequest};

#[get("/")]
async fn index(req: HttpRequest) -> impl Responder {
    Inertia::back(&req)
}
```

If you've configured your [Inertia Temporary Session] middleware, it will use the previous visited page with
more accuracy. Otherwise, it'll look for a `referer` header. In last case, it will redirect to `/`.

Redirecting back with props is a little bit of advanced. Refer to [Redirecting Back With Errors] for an
detailed explanation.

[Inertia Temporary Session]: ../advanced/flash-messages.md
[Redirecting Back With Errors]: ../advanced/flash-messages.md#redirecting-back-with-errors

## Responses
Calling the facade `render` method render a page (be it ReactJs, Svelte or Vue.js):

```rust
use inertia_rust::{Inertia, InertiaFacade};
use actix_web::{get, Responder, HttpRequest};

#[get("/users")]
async fn index(req: HttpRequest) -> impl Responder {
    Inertia::render(&req, "Users/Index".into()).await
}
```

`"Users/Index"` is a `Component` --- which is just a meaningful struct containing the name of the jsx, vue or
svelte component to be rendered. Naturally, you must have configured your `www/app.ts` file to find this
page component (again, remember to check [Inertia.js documentation]).

For example, it could be:

```jsx
import { Head } from "@inertiajs/react";

type User = {
    name: string;
    email: string;
}

type UsersPageProps = {
    users: User[];
}

export default function UsersPage({ user }: UsersPageProps) {
    return (
        <>
            <Head title="Users" />
            <div>
                <h1>Users</h1>
                {users.map(user => (
                    <div>
                        <span><strong>Name: </strong>{user.name}</span>
                        <span><strong>E-mail: </strong>{user.email}</span>
                    </div>
                ))}
            </div>
        </>
    )
}
```

You've noticed that this page has a `user` property. Actually, Inertia Rust is the one who will provide
this property! We didn't provide any props to the page, though. We'll fix this in the next chapter, where
props are discussed.

[Inertia.js documentation]: https://inertia-rails.dev/guide/client-side-setup

## Root Template Data

Depending on the template resolver you're using, there are different manners of passing data directly to
the root template.

The built-in `ViteTemplateResolver` allows you to achieve this through request's view data:

```rust
use inertia_rust::{hashmap, Inertia, InertiaFacade};
use actix_web::{get, Responder, HttpRequest};

#[get("/")]
async fn index(req: HttpRequest) -> impl Responder {
    Inertia::view_data(&req, hashmap![ "meta" => "Your page description".into() ]);
    Inertia::render(&req, "Index".into()).await
}
```

After calling the `view_data` method, you can access the defined data in your template HTML the following
way:

```html
<!-- using the deprecated ViteTemplateResolver, in a .html file -->
<meta name="description" content="@inertia::view_data(meta)">
```

```hbs
{{!-- using the ViteHBSTemplateResolver, in a .hbs file --}}
<meta name="description" content="{{ view_data.meta }}">
```

If the value isn't defined, ViteTemplateResolver replaces the value by an ordinary JavaSript `null`.

> Remember: a different template resolver could allow you to pass data in different ways, for example,
> through the actual page props. You should check the template resolver documentation for more details.

### Default View Data
`ViteTemplateResolver` will always inject a default view data property: `isSsr`. You can use it, for
instance, to conditionally hydrate or create your React root. This property will be true only when
the Inertia response has been server-side rendered.
