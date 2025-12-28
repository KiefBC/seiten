# Seiten

A self-hosted tool for managing anime canon/filler collections in Plex, inspired by the *arr stack.

## Overview

Seiten scrapes episode data from [AnimeFillerList](https://www.animefillerlist.com), caches it locally, and creates Plex collections containing only canon episodes. This lets you skip filler arcs without manually cross-referencing episode guides.

---

## Goals

### Phase 1 — Manual Management

#### 1.1 Project Setup ✅
- [x] Initialize Cargo workspace
- [x] Create `core`, `migration`, and `app` crates
- [x] Configure workspace dependencies
- [ ] Set up Tailwind CSS
- [x] Verify Leptos dev server runs

#### 1.2 Database Layer
- [x] Install `sea-orm-cli`
- [x] Write initial migration (shows, episodes, show_mappings, sync_log)
- [x] Run migration, generate entities 
- [ ] Create database connection helper
- [ ] Write basic queries: insert show, get show by slug, get episodes

#### 1.3 AnimeFillerList Scraper
- [ ] Set up reqwest client with user agent
- [ ] Implement search endpoint parsing
- [ ] Implement show page scraping (episode table)
- [ ] Parse episode types from CSS classes
- [ ] Handle edge cases (missing data, different page layouts)
- [ ] Write scraped data to database

#### 1.4 Plex API Client
- [ ] Implement authentication (token-based) - structure only
- [ ] Get library sections - structure only
- [ ] Get shows in a library - structure only
- [ ] Get episodes for a show - structure only
- [ ] Create collection - structure only
- [ ] Add items to collection - structure only

#### 1.5 UI — Shell (Basic Setup)
- [x] Set up Leptos app with router
- [ ] Configure Thaw UI theme
- [ ] Create layout component (nav, content area)
- [ ] Add placeholder routes: Home, Search, Library, Settings

#### 1.6 UI — Search & Scrape
- [ ] Build search input with debounce (leptos-use)
- [ ] Display search results as cards/list
- [ ] Add "Import" button per result
- [ ] Show scrape progress/status
- [ ] Display success/error feedback

#### 1.7 UI — Plex Connection
- [ ] Settings page for Plex URL and token
- [ ] Persist to localStorage (leptos-use)
- [ ] Test connection button
- [ ] Library selector dropdown
- [ ] Display shows from selected library

#### 1.8 UI — Episode Viewer
- [ ] List cached shows
- [ ] Show detail page with episode table
- [ ] Color-code by episode type
- [ ] Filter toggles: Canon / Mixed / Filler / Anime Canon
- [ ] Episode count summary

#### 1.9 UI — Collection Creation
- [ ] Match cached show to Plex show (manual selection)
- [ ] Preview which episodes will be in collection
- [ ] Collection naming input
- [ ] Create collection button
- [ ] Success/error feedback

### Phase 2 — Automatic Sync

#### 2.1 Show Mapping Persistence
- [ ] Save manual show-to-Plex mappings in database
- [ ] UI to view/edit existing mappings
- [ ] Delete mapping option

#### 2.2 Auto-Match Logic
- [ ] Fuzzy title matching (core crate)
- [ ] Confidence scoring
- [ ] Suggest matches for unmapped shows
- [ ] UI to confirm/reject suggestions

#### 2.3 Sync Engine
- [ ] Batch process all mapped shows
- [ ] Compare cached episodes vs Plex collection
- [ ] Add missing episodes to collection
- [ ] Remove filler if present (optional setting)
- [ ] Handle errors gracefully, continue with next show

#### 2.4 Sync UI
- [ ] "Sync All" button
- [ ] Progress indicator (X of Y shows)
- [ ] Live log output
- [ ] Summary on completion

#### 2.5 Sync History
- [ ] Write sync results to sync_log table
- [ ] History page showing past syncs
- [ ] Per-show sync status (last synced, episodes synced)

#### 2.6 Scheduled Sync (Optional)
- [ ] Background task with configurable interval
- [ ] Enable/disable toggle in settings
- [ ] Last run timestamp display

### Phase 3 — Future Enhancements (Backlog)
- [ ] AniDB cross-referencing for better matching
- [ ] Multiple collection strategies per show
- [ ] Sonarr/Radarr-style activity feed
- [ ] Webhook notifications
- [ ] Multi-user support
- [ ] Backup/restore configuration

## Tech Stack

| Layer | Technology |
|-------|------------|
| Language | Rust |
| Frontend | Leptos (SSR with hydration) |
| Backend | Leptos + Axum |
| Database | SQLite via SeaORM |
| HTTP Client | reqwest |
| HTML Parsing | scraper |
| Styling | Tailwind CSS |
| UI Components | Thaw UI |
| Utilities | Leptos-Use |
| Deployment | Docker |

### Core Crate

Handles all business logic independent of the UI:

- **entities/** — SeaORM entities: `Series`, `Episode`, `EpisodeType` (Canon, MixedCanon, Filler, AnimeCanon)
- **server/db.rs** — Database connection and query helpers
- **server/scraper.rs** — Fetches and parses AnimeFillerList pages
- **server/plex.rs** — Plex API client for libraries, shows, episodes, and collections

### Migration Crate

SeaORM migrations for schema management. Run with `sea-orm-cli migrate`.

### App Crate

Leptos SSR application:

- Server-rendered HTML with client-side hydration
- Server functions for data operations (no separate REST API)
- Type-safe across frontend and backend

## Data Model

```sql
shows (id, slug, title, last_fetched)
episodes (id, show_id, episode_num, episode_type, title)
show_mappings (id, show_id, plex_rating_key, plex_title)
sync_log (id, show_id, synced_at, episodes_synced)
```

### Episode Types

| Type | Description |
|------|-------------|
| Canon | Manga canon — directly adapts source material |
| MixedCanon | Partially canon with some filler content |
| AnimeCanon | Original content considered canon by the studio |
| Filler | Non-canon, skippable |

## Deployment

Runs as a Docker container alongside other *arr tools:

```yaml
services:
  seiten:
    image: seiten:latest
    container_name: seiten
    restart: unless-stopped
    ports:
      - "8080:8080"
    volumes:
      - ./config:/config
    environment:
      - PLEX_URL=${PLEX_URL}
      - PLEX_TOKEN=${PLEX_TOKEN}
```

Single binary with embedded assets — no Node runtime required.

## Dependencies

### Core
- `tokio` — async runtime
- `reqwest` — HTTP client
- `scraper` — HTML parsing
- `sea-orm` — async ORM with SQLite backend
- `sea-orm-migration` — database migrations
- `serde` / `serde_json` — serialization
- `chrono` — timestamps
- `anyhow` / `thiserror` — error handling

### App
- `leptos` — reactive UI framework
- `leptos_axum` — server integration
- `leptos_router` — client-side routing
- `thaw` — UI component library (buttons, forms, modals, tables)
- `leptos-use` — reactive utilities (debounce, localStorage, click outside, etc.)
- `tower-http` — middleware (CORS, etc.)

## Known Challenges

1. **Show matching** — AnimeFillerList slugs don't always match Plex metadata. Will need fuzzy matching or manual mapping.

2. **HTML scraping fragility** — No official API; dependent on AnimeFillerList's HTML structure. Cache aggressively to minimize requests.

3. **Multi-season handling** — Some shows split seasons differently between AnimeFillerList and Plex. May need episode offset configuration.

## References

- [AnimeFillerList](https://www.animefillerlist.com)
- [Plex API Documentation](https://plexapi.dev)
- [Leptos Book](https://book.leptos.dev)
- [SeaORM Documentation](https://www.sea-ql.org/SeaORM/docs/index)
- [Thaw UI](https://thawui.vercel.app)
- [Leptos-Use](https://leptos-use.rs)
