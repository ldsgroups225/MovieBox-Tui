# External API Integrations

This page records the network interfaces MovieBox-TUI currently calls, as implemented in this repository. Provider sites and private endpoints can change without notice. The routes below describe observed client behavior; they are not guarantees from the service operators.

## At a glance

| Integration | Interface | Transport | Authentication in this client |
| :--- | :--- | :--- | :--- |
| MovieBox | Mobile BFF JSON endpoints | HTTPS | Visitor bearer token and generated request-signing headers |
| Dramachi | Provider-specific JSON query interface | HTTPS | None sent by this client |
| 4KHDHub | HTML pages and mirror resolvers | HTTPS | Browser-style user agent; no provider login |
| CircleFTP | Posts JSON API and uploaded media | HTTP | None sent by this client |
| DhakaFlix | Private-network JSON file index | HTTP | None sent by this client |
| Stremio add-ons | Add-on manifest and resource protocol | HTTP or HTTPS | No generic add-on credentials or custom headers |
| TV playlists | User-supplied M3U files or URLs | Local file, HTTP, or HTTPS | No configurable auth headers; the supplied URL is fetched as given |
| Updater | GitHub Releases API and assets | HTTPS | No GitHub token |

## MovieBox

**Implementation:** `src/providers/moviebox/client.rs`, `src/providers/moviebox/mod.rs`, `src/providers/moviebox/crypto.rs`.

MovieBox uses a reverse-engineered mobile BFF, not a documented public API contract. The client tries these HTTPS hosts in order and advances to another host after network, parsing, or selected retryable HTTP failures:

```text
https://api6.aoneroom.com
https://api5.aoneroom.com
https://api4.aoneroom.com
https://api4sg.aoneroom.com
https://api3.aoneroom.com
https://api6sg.aoneroom.com
https://api.inmoviebox.com
```

| Method | Path | Purpose |
| :--- | :--- | :--- |
| `POST` | `/wefeed-mobile-bff/user-api/visitor-login` | Obtain a visitor session; request body is `{}`. |
| `POST` | `/wefeed-mobile-bff/subject-api/search/v2` | Search with JSON fields `keyword`, `page`, `perPage` (15), and `subjectType` (0). |
| `GET` | `/wefeed-mobile-bff/subject-api/get?subjectId={id}` | Fetch title details. |
| `GET` | `/wefeed-mobile-bff/subject-api/season-info?subjectId={id}` | Fetch series seasons. |
| `GET` | `/wefeed-mobile-bff/subject-api/play-info/v2?subjectId={id}[&se={season}&ep={episode}]` | Fetch playback information. |
| `GET` | `/wefeed-mobile-bff/subject-api/resource?subjectId={id}[&se={season}&ep={episode}]&page={page}&perPage={count}[&resolution={value}]` | List stream resources and available resolutions. |
| `GET` | `/wefeed-mobile-bff/subject-api/get-ext-captions?subjectId={id}&resourceId={id}` | Fetch external caption metadata. |
| `GET` | `/wefeed-mobile-bff/tab-operating?page={page}&tabId={id}&version=` | Fetch homepage tab content. |

Authenticated requests include a bearer visitor token and generated headers such as `x-client-token`, `x-tr-signature`, and `x-client-info`, plus locale headers `region` and `lang`. `MOVIEBOX_LOCALE` defaults to French (`fr`/`FR`) and accepts English (`en`/`US`). The client reuses a locally persisted visitor token when valid and refreshes it after HTTP 401/403 or host-pool exhaustion. It rotates hosts on network/parsing errors and HTTP 403, 406, 407, 429, 500, 502, 503, or 504. Successful JSON responses are unwrapped from a top-level `data` property when present. These details are implementation-specific and may stop working if MovieBox changes its mobile client protocol.

## Dramachi

**Implementation:** `src/providers/dramachi/client.rs`, `src/providers/dramachi/models.rs`.

The default JSON endpoint is `https://api.nodeobjects.com/`. Requests use query parameters on the root URL:

| Query | Purpose |
| :--- | :--- |
| `interface=search&q={query}&filter=all&page={page}` | Search titles; the client reads results from `data`. |
| `interface=title_v2&id={id}` | Fetch title details. |
| `interface=eplist&season={season}&id={id}` | Fetch and sort episode entries. |
| `interface=getFile&fid={file_id}&findex={disk}` | Resolve an episode file; the response supplies host/file data used to construct an HTTPS `/cdn/` stream URL. |

Poster paths are prefixed with `https://static.nodeobjects.com/thumbnail/`. The client sends no API key or login credentials. The response schema and routes are provider-specific; they are not a stable standard API.

## 4KHDHub

**Implementation:** `src/providers/fourkhdhub/client.rs`, `src/providers/fourkhdhub/parser.rs`, `src/providers/fourkhdhub/hubcloud.rs`.

The default site is `https://4khdhub.one/`. `MOVIEBOX_FOURKHDHUB_URL` can replace the base URL, but the client requires it to use HTTPS.

This integration does not call a JSON API. Search sends an HTTPS `GET` to the site with the `s={query}` parameter; search, detail, and release data are extracted from HTML pages. Playback then follows scraped mirror links. HubCloud, HubDrive, and GreenMotors links have provider-specific resolvers; other links are validated as candidate playback URLs and probed before use. These page layouts and resolver flows are unofficial and can break when the site changes.

## CircleFTP (BDIX)

**Implementation:** `src/providers/bdix/circleftp/client.rs`, `src/providers/bdix/circleftp/parser.rs`.

The client uses plain HTTP at `http://new.circleftp.net:5000`:

| Method | Path | Purpose |
| :--- | :--- | :--- |
| `GET` | `/api/posts?searchTerm={query}&order=desc` | Search; the client reads the `posts` array. |
| `GET` | `/api/posts/{id}` | Fetch one post's details and release links. |
| `HEAD` | `{release_url}` | Best-effort media size lookup. |
| `GET` | `/uploads/{filename}` | Load post artwork. |

Movie releases are read from a post's `content` link; series releases are read from episode `link` fields inside `content`. No authentication is sent by the client. This provider is intended for networks that can reach the CircleFTP BDIX host.

## DhakaFlix (BDIX)

**Implementation:** `src/providers/bdix/dhakaflix/client.rs`.

DhakaFlix is queried over HTTP at private-network addresses. The built-in server roots are:

```text
http://172.16.50.7/DHAKA-FLIX-7/
http://172.16.50.14/DHAKA-FLIX-14/
http://172.16.50.12/DHAKA-FLIX-12/
http://172.16.50.9/DHAKA-FLIX-9/
```

The client sends JSON `POST` requests to each root. Search uses:

```json
{"action":"get","search":{"href":"/DHAKA-FLIX-7/","pattern":"query","ignorecase":true}}
```

The response's `search` array contains `href` entries. To list a result folder, the client sends `{"action":"get","items":{"href":"{path}","what":1}}`; it reads the returned `items` array (`href`, `size`) and builds stream URLs from the server address plus each file path. There is no authentication in the client. These RFC1918 addresses are reachable only from a network with routes to that private BDIX network.

## Stremio add-ons

**Implementation:** `src/providers/addons/client.rs`, `src/providers/addons/aggregator.rs`, `src/providers/addons/models.rs`.

Add-on URLs are normalized to a `/manifest.json` URL. A `stremio://` URL is changed to HTTPS; a URL without a scheme is also given HTTPS. The built-in Cinemeta manifest is `https://v3-cinemeta.strem.io/manifest.json`.

The client reads the manifest's resources, types, catalogs, and ID prefixes, then calls these HTTP resources:

| Method | Route | Purpose |
| :--- | :--- | :--- |
| `GET` | `/manifest.json` | Load the add-on manifest. |
| `GET` | `/catalog/{type}/{catalogId}.json` | Load a catalog. |
| `GET` | `/catalog/{type}/{catalogId}/{extra}.json` | Load a catalog page/filter. |
| `GET` | `/catalog/{type}/{catalogId}/search={query}.json` | Search a catalog. |
| `GET` | `/meta/{type}/{id}.json` | Load item metadata. |
| `GET` | `/stream/{type}/{id}.json` | Load streams. |

Catalog responses may be a raw array or an object containing `metas`, `items`, or `results`; stream responses may be a raw array or an object containing `streams`. Series stream IDs are sent as `{id}:{season}:{episode}` with type `series`; movie requests use type `movie`. Only enabled add-ons advertising the `stream` resource are queried for playback, concurrently, with a five-second per-add-on timeout. The client merges duplicate direct URLs and sorts results by quality/size.

The generic client does not send add-on-specific authentication credentials or custom request headers. It does not call the separate Stremio `/subtitles/...` resource. MovieBox-TUI's add-on routes follow the [Stremio Add-on Protocol](https://github.com/Stremio/stremio-addon-sdk/blob/master/docs/protocol.md), with the subset listed above.

## Live TV playlists

**Implementation:** `src/providers/tv/parser.rs`, `src/providers/tv/models.rs`.

TV mode reads a user-provided local file or fetches a user-provided HTTP(S) URL. It parses the common M3U `#EXTINF` playlist format. For each channel it uses `tvg-id`, `tvg-logo`, and `group-title` attributes, the text after the `#EXTINF` comma as the channel name, and the following non-comment line as the stream URL. The playlist loader caches remote source files for 24 hours and applies a 15 MiB check to local files and normal remote loads. Remote text is materialized before the length check; the cache-read recovery path also re-downloads without repeating the size check.

This parser does not fetch or parse XMLTV data and does not implement EPG lookup. `tvg-id` is retained as channel metadata only. Although the README has previously described EPG support, no EPG endpoint or XMLTV parser is present in the current source.

The playlist URL is the only catalog input; the selected channel's stream URL is passed to the configured external player. The playlist fetch uses the shared HTTP client and does not add provider-specific headers or credentials.

## GitHub release updater

**Implementation:** `src/updater/check.rs`, `src/updater/artifact.rs`, `src/updater/download.rs`, `src/updater/verify.rs`.

The updater checks `GET https://api.github.com/repos/mesamirh/MovieBox-Tui/releases/latest` and reads `tag_name`, `body`, and `assets[]` (`name`, `browser_download_url`, `size`). If the API request fails or is rate-limited, it follows `https://github.com/mesamirh/MovieBox-Tui/releases/latest` and reads the redirected release tag. It downloads the platform archive and `SHA256SUMS` over HTTPS, then verifies the archive checksum locally. No GitHub token is sent. The self-updater does not select a binary asset for Termux/Android.

## Network behavior and limits

HTTP requests share the networking helpers in `src/net.rs`, which configure DNS resolution and common connection-pool defaults. Providers override timeouts and user agents where needed. The updater uses HTTPS for release metadata and downloaded artifacts; the BDIX integrations intentionally use HTTP/private-network hosts, while user-supplied playlist URLs and add-on URLs may use HTTP or HTTPS.

Provider routes, private hosts, response shapes, and HTML selectors are read from the current source. Treat them as integration details that may require code updates when upstream services change.
