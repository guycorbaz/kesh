# Prompt — passe 2 de `bmad-create-story validate`, Story 25-1b

*Versionné le 2026-09-11. Lentille unique (Opus 5), contexte frais, après la remédiation
de la passe 1.*

Tu es un **valideur adversarial** en contexte frais. Ton objet est
`_bmad-output/implementation-artifacts/25-1b-trous-alimentation.md`, dépôt
`/home/gcorbaz/devel/kesh`.

## Ce que cette passe cherche EN PRIORITÉ

⛔ **La passe 1 a produit sept patches, et c'est là que ce dépôt trouve ses défauts.** Le motif est
mesuré et écrit dans son `CLAUDE.md` : *sur huit passes cumulées de deux stories récentes, SEPT ont
trouvé une régression du patch précédent et AUCUNE un défaut de la conception d'origine.* Lis donc
le **dernier commit** (`git show b2c31537`) et interroge d'abord ce qu'il a changé.

Les sept patches, à reprendre un par un :

1. **AC 3 + tableau des Dev Notes + T4** — l'extraction de `imported_supplier_invoices::create_in_tx`
   et la transaction portée par `process_one_file`. ⚠️ **Est-ce seulement faisable ?** Qui appelle
   `create` ? Le verrou `GET_LOCK` de `run_inbox_import` interfère-t-il avec une transaction ouverte
   plus bas ? Une transaction par fichier est-elle tenable quand la boucle en traite N, et que se
   passe-t-il si l'une échoue au milieu ? **Le patch de la passe 1 peut avoir créé un problème que
   la passe 1 ne pouvait pas voir.**
2. **AC 11** — le diff ensembliste par identité. Est-il réalisable sur `lib.rs` tel qu'il est
   écrit ? Une route y est-elle identifiable de façon **stable et unique** (verbe + handler) ?
   Deux routes peuvent-elles partager un handler ? Le motif d'extraction proposé résiste-t-il à un
   `.route()` écrit sur plusieurs lignes, ou à un `nest()` ?
3. **AC 12** — les trois sites du manuel. **Vérifie les trois au sol**, y compris que les numéros de
   ligne sont exacts et que le contenu cité l'est mot pour mot. ⚠️ Et cherche un **quatrième** site :
   sur la story sœur 25-1a, l'inventaire des documents publiés a été pris en défaut **cinq fois**,
   dont une par la seule **langue** du support. Contrôle le **PDF aplati**, pas seulement le `.tex`.
4. **Les deux références de ligne corrigées** (`companies.rs`, `audit.rs`) — sont-elles justes
   maintenant ?
5. **Les décomptes** — la spec annonce **12 AC** et **11 tâches**. Recompte depuis la source. Tout
   nombre écrit ailleurs dans le document doit être cohérent avec sa propre ventilation.

## Puis, le reste de la spec

Sans t'y limiter : la faisabilité des autres extractions `_in_tx`, la décidabilité des invariants du
volet B, l'exactitude des références restantes, et **tout ce qui manque** — une route, un appelant,
un mode d'échec que ni la spécification ni la passe 1 n'ont vu.

⛔ **Rien ne se croit sur parole.** Chaque affirmation se vérifie dans le code avant d'être retenue
ou contestée.

## Ce que tu rends

- **Les findings** : sévérité (`CRITICAL` / `HIGH` / `MEDIUM` / `LOW`), l'endroit exact, **la
  commande ou l'extrait qui l'établit**, et ce qu'il faut changer.
- **Pour chacun, dis s'il est né d'un patch de la passe 1 ou s'il est d'origine.** C'est cette
  ventilation qui dit si la boucle converge.
- ⛔ **La liste des axes réellement exercés ET de ceux qui ne l'ont pas été.** Un « 0 finding » non
  adossé à cette liste ne clôt rien et ne compte pas comme passe. Ne qualifie jamais de « robuste »
  un point que tu n'as pas exécuté.

## Interdits

⛔ **N'écris aucun fichier et n'exécute aucune commande qui écrit** — nommément
`scripts/prepare-release.sh`, `scripts/install-hooks.sh`, tout `git commit`/`push`/`checkout`/`add`,
`sqlx migrate`, et toute écriture en base. Lire et compiler pour vérifier, rien de plus.
