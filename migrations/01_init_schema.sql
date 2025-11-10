-- Manga Scraper Database Schema for PostgreSQL
-- This schema is automatically applied when the PostgreSQL container starts

-- Enable UUID extension for UUID generation
CREATE EXTENSION IF NOT EXISTS "uuid-ossp";

-- Sources table: stores manga source websites
CREATE TABLE IF NOT EXISTS sources (
    id SERIAL PRIMARY KEY,
    name VARCHAR(255) NOT NULL UNIQUE,
    url TEXT NOT NULL
);

-- Manga table: stores manga metadata
CREATE TABLE IF NOT EXISTS manga (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    title TEXT NOT NULL,
    alt_titles TEXT,
    cover_url TEXT,
    description TEXT,
    tags TEXT,
    rating VARCHAR(50),
    monitored INTEGER,
    check_interval_secs INTEGER,
    discover_interval_secs INTEGER,
    last_chapter_check BIGINT,
    last_discover_check BIGINT,
    mal_id INTEGER,
    anilist_id INTEGER,
    mangabaka_id VARCHAR(255)
);

-- Manga source data: maps manga to their sources
CREATE TABLE IF NOT EXISTS manga_source_data (
    id SERIAL PRIMARY KEY,
    manga_id UUID NOT NULL,
    source_id INTEGER NOT NULL,
    source_manga_id TEXT NOT NULL,
    source_manga_url TEXT NOT NULL,
    FOREIGN KEY (manga_id) REFERENCES manga (id) ON DELETE CASCADE,
    FOREIGN KEY (source_id) REFERENCES sources (id) ON DELETE CASCADE,
    UNIQUE(manga_id, source_id)
);

-- Chapters table: stores chapter information
CREATE TABLE IF NOT EXISTS chapters (
    id SERIAL PRIMARY KEY,
    manga_source_data_id INTEGER NOT NULL,
    chapter_number TEXT NOT NULL,
    url TEXT NOT NULL,
    scraped BOOLEAN NOT NULL DEFAULT FALSE,
    FOREIGN KEY (manga_source_data_id) REFERENCES manga_source_data (id) ON DELETE CASCADE,
    UNIQUE(manga_source_data_id, url)
);

-- Provider IDs mapping: external metadata provider IDs
CREATE TABLE IF NOT EXISTS provider_ids (
    id SERIAL PRIMARY KEY,
    manga_id UUID NOT NULL,
    provider VARCHAR(100) NOT NULL,
    provider_id VARCHAR(255) NOT NULL,
    UNIQUE(manga_id, provider),
    FOREIGN KEY (manga_id) REFERENCES manga(id) ON DELETE CASCADE
);

-- Indexes for performance
CREATE INDEX IF NOT EXISTS idx_msd_manga ON manga_source_data(manga_id);
CREATE INDEX IF NOT EXISTS idx_msd_source ON manga_source_data(source_id);
CREATE INDEX IF NOT EXISTS idx_provider_manga ON provider_ids(manga_id);
CREATE INDEX IF NOT EXISTS idx_ch_msd ON chapters(manga_source_data_id);
CREATE INDEX IF NOT EXISTS idx_manga_title ON manga(title);

-- Seed data: insert default sources
-- This matches the Source enum values from the Rust code
INSERT INTO sources (id, name, url) VALUES
    (1, 'MangaDex', 'https://mangadex.org'),
    (2, 'Mangakakalot', 'https://mangakakalot.com'),
    (3, 'Anilist', 'https://anilist.co'),
    (4, 'MyAnimeList', 'https://myanimelist.net'),
    (5, 'Fanfox', 'https://fanfox.net'),
    (6, 'MangaPark', 'https://mangapark.net'),
    (7, 'FireScans', 'https://fr-scan.com'),
    (8, 'RizzComic', 'https://rizzfables.com'),
    (9, 'ResetScans', 'https://reset-scans.us'),
    (10, 'AsuraScans', 'https://asuratoon.com'),
    (11, 'Mangabaka', 'https://mangabaka.com'),
    (12, 'VoidScans', 'https://void-scans.com'),
    (13, 'NitroScans', 'https://nitroscans.com'),
    (14, 'LuminousScans', 'https://luminousscans.com'),
    (15, 'AkumaScans', 'https://akumascans.com'),
    (16, 'DayComics', 'https://daycomics.com'),
    (17, 'ReaperScans', 'https://reaperscans.com'),
    (18, 'FlameComics', 'https://flamecomics.com'),
    (19, 'HiveToons', 'https://hivetoons.org'),
    (20, 'KogaScans', 'https://kogascans.com'),
    (21, 'KenScans', 'https://kenscans.com'),
    (22, 'YokaiJump', 'https://yokaijump.fr'),
    (23, 'MirageScans', 'https://miragescans.com'),
    (24, 'ManhuaUS', 'https://manhuaus.com'),
    (25, 'ZeroScans', 'https://zscans.com'),
    (26, 'Utoon', 'https://utoon.net'),
    (27, 'KodokuStudio', 'https://kodokustudio.com'),
    (28, 'PrismScans', 'https://prismascans.com'),
    (29, 'LunaToons', 'https://lunatoons.com'),
    (30, 'CosmicScans', 'https://cosmic-scans.com'),
    (31, 'NyxScans', 'https://nyxscans.com'),
    (32, 'VoidScansV2', 'https://void-scans.com'),
    (33, 'QiScans', 'https://qiscans.com'),
    (34, 'RizzFables', 'https://rizzfables.com'),
    (35, 'RokariComics', 'https://rokaricomics.com'),
    (36, 'MiracleScans', 'https://miraclescans.com'),
    (37, 'SirenScans', 'https://sirenscans.com'),
    (38, 'StoneScape', 'https://stonescape.xyz'),
    (39, 'AsmoToon', 'https://asmotoon.com'),
    (40, 'FreakScans', 'https://freakscans.com'),
    (41, 'TempleScan', 'https://templetoons.com'),
    (42, 'ThunderScans', 'https://thunderscans.com'),
    (43, 'DrakeComic', 'https://drakecomic.com'),
    (44, 'WitchScans', 'https://witchscans.com'),
    (45, 'VortexScans', 'https://vortexscans.org'),
    (46, 'MadaraScans', 'https://madaradex.org'),
    (47, 'Tapas', 'https://tapas.io'),
    (48, 'KaganeScans', 'https://kaganescans.com'),
    (49, 'KDTNovels', 'https://kdt-novels.com'),
    (50, 'RizzleFlix', 'https://rizzleflix.com'),
    (51, 'EclipseScans', 'https://eclipsescans.com'),
    (52, 'VerseScans', 'https://versescans.com'),
    (53, 'FreeScanlation', 'https://free-scanlation.com'),
    (54, 'VastVisual', 'https://vastvisual.com'),
    (55, 'MangaQueen', 'https://mangaqueen.com'),
    (56, 'OmegaScans', 'https://omegascans.org'),
    (57, 'ApolloComics', 'https://apollocomics.com'),
    (58, 'Webtoon', 'https://www.webtoons.com'),
    (59, 'ArtemisScans', 'https://artemisscans.com'),
    (60, 'TitanManga', 'https://titanmanga.com'),
    (61, 'MavinTranslations', 'https://mavintranslations.com')
ON CONFLICT (id) DO NOTHING;

-- Reset sequence to continue from highest ID
SELECT setval('sources_id_seq', (SELECT MAX(id) FROM sources));
