<div align="center">

# Seiten (正典)

A self-hosted tool for managing anime canon/filler collections in Plex. Seiten scrapes episode data from AnimeFillerList, caches it locally, and creates Plex collections containing only canon episodes.

<img src="Example.png" alt="Seiten Screenshot" />

**Built with:** Rust, Leptos (SSR), Axum, SeaORM, Tailwind CSS, and DaisyUI

</div>

## Running the Development Server

```bash
npm install
npm run dev
```

This runs both Tailwind CSS watch and cargo-leptos watch concurrently. The app will be available at `http://127.0.0.1:3000`

## Building for Production

```bash
npm run tailwind:build
cargo leptos build --release
```

This generates:
- Server binary in `target/server/release`
- Site package in `target/site`

## Configuration

Create a `.env` file in the project root (see `.env.example`):

```env
DATABASE_URL=sqlite://db.sqlite?mode=rwc

# Get these credentials at: https://anidb.net/perl-bin/animedb.pl?show=client
ANIDB_CLIENT_ID=seite
ANIDB_CLIENT_VERSION=1

PLEX_URL=ip:port
PLEX_TOKEN=your-plex-token
```

## Areas where AI helped

- Generating a starting point for the README.md file.
- Helped integrate Tailwind CSS with Leptos.
- Assisted in general Cargo.toml dependency management.
- Playwright integration for end-to-end testing.

## License

MIT
