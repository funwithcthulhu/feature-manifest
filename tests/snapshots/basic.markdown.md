# feature-manifest-fixture feature manifest

Default feature set: `serde`

| Feature | Default | Visibility | Status | Enables | Description |
| --- | --- | --- | --- | --- | --- |
| `docs-preview` | no | public | stable | `serde`, `tokio?/rt` | Generates docs \| preview output.<br>Includes async examples. Note: Escapes table cells and Mermaid labels. |
| `native-tls` | no | public | deprecated | — | Use the system TLS stack. |
| `rustls` | no | public | stable | — | Use rustls for TLS. |
| `serde` | yes | public | stable | `dep:serde` | Enables Serialize/Deserialize impls. |
| `std` | no | public | stable | — | Enables the standard library surface. |
| `tokio` | no | public | stable | `dep:tokio`, `std` | Enables async APIs backed by Tokio. |
| `unstable` | no | public | unstable | — | Experimental APIs; semver not guaranteed. |

_1 internal/private feature(s) hidden. Use `--include-private` to render all._

## Groups

- `tls`: Select one TLS backend. Mutually exclusive. Members: `rustls`, `native-tls`.
