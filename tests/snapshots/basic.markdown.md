# feature-manifest-fixture feature manifest

Default feature set: `serde`

| Feature | Default | Visibility | Status | Category | Enables | Description |
| --- | --- | --- | --- | --- | --- | --- |
| `docs-preview` | no | public | stable | docs | `serde`, `tokio?/rt` | Generates docs \| preview output.<br>Includes async examples. Note: Escapes table cells and Mermaid labels. Since: 0.2.0 Docs: https://docs.rs/feature-manifest Tracking issue: https://github.com/funwithcthulhu/feature-manifest/issues/1 Requires: serde |
| `native-tls` | no | public | deprecated | — | — | Use the system TLS stack. |
| `rustls` | no | public | stable | — | — | Use rustls for TLS. |
| `serde` | yes | public | stable | serialization | `dep:serde` | Enables Serialize/Deserialize impls. Since: 0.1.0 Docs: https://docs.rs/serde |
| `std` | no | public | stable | — | — | Enables the standard library surface. |
| `tokio` | no | public | stable | — | `dep:tokio`, `std` | Enables async APIs backed by Tokio. |
| `unstable` | no | public | unstable | — | — | Experimental APIs; semver not guaranteed. |

_1 internal/private feature(s) hidden. Use `--include-private` to render all._

## Groups

- `tls`: Select one TLS backend. Mutually exclusive. Members: `rustls`, `native-tls`.
