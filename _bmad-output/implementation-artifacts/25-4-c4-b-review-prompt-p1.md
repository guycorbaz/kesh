# Prompt — revue de code P1, Story 25-4-c4-b (l'arrondi à 5 centimes, visible et réglable)

*Versionné le 2026-10-01. Trois lentilles (Sonnet), contexte frais chacune.*

Dépôt `/home/gcorbaz/devel/kesh`, branche `story/25-4-c4-arrondi-facture-5-centimes`. **Diff à revoir :
`git diff becfe6b2 5ea143cc`** (un seul commit d'implémentation, aplati). Fiche :
`_bmad-output/implementation-artifacts/25-4-c4-b-arrondi-visible-et-reglage.md` (AC 1–6). Sœur, déjà faite :
`25-4-c4-a-arrondi-fige-a-la-validation.md`. CR : `gh issue view 494`. Règles : `CLAUDE.md`.

## Lentilles

- **A — Blind hunter** : le diff seul. PDF (`crates/kesh-qrbill/src/pdf.rs` : `recap_lines`, réserve
  `TooManyLines`, `format_signed_ch` sur un zéro ou un négatif, alignement de `I18N_KEYS`/`DEFAULT_EN`),
  `with_rounding_preview` (statut testé par chaîne, TTC d'un brouillon), big.js à la fiche, sérialisation.
- **B — Edge-case hunter** : diff + code environnant. Facture annulée (`cancelled`) ; avoir et rappel au PDF ;
  brouillon sans ligne ; réglage changé entre lecture et affichage ; `get_or_create_default` qui ÉCRIT sur un GET
  (concurrence, verrou, coût sur chaque lecture de brouillon) ; clés d'API en lecture seule qui lisent une facture
  brouillon (le `GET` reste-t-il en lecture ?) ; formulaire des réglages envoyé par un client ancien sans le champ ;
  E2E : la spec neuve laisse-t-elle un état qui pollue les autres ?
- **C — Acceptance auditor** : diff + fiche. Chaque AC tenu ? **Pars du symptôme** : `grep -rn "totalTtc\|total_ttc\|Total TTC\|invoice-pdf-total" frontend/src crates/` — un écran ou un document montre-t-il encore un total arrondi SANS sa ligne d'arrondi (liste des factures, échéancier, avoir à l'écran, e-mail) ? Les tests prouvent-ils ce qu'ils disent (rouges avant le patch) ? i18n (4 locales), `sitesTotal`, garde des sélecteurs E2E. **Le manuel** : `docs/manual/fr/{user,admin}-manual.tex` dit-il vrai contre le code — **PDF aplati** (`pdftotext … | tr '\n' ' ' | tr -s ' '`, ligatures ﬀ/ﬁ) vers `/tmp/claude-1000/-home-gcorbaz-devel-kesh/379e6f94-8029-42cb-9720-fa27c2fb204c/scratchpad/`. CHANGELOG.

## Ce que tu rends

- **Findings** : sévérité, `fichier:ligne`, **preuve** (commande et sortie, ou code cité relu), scénario d'échec,
  correction. Pour tout finding affirmant qu'un code est absent ou présent : la sortie d'un `grep -nF`.
- ⛔ **La liste des axes réellement exercés ET de ceux qui ne l'ont pas été.**

## Interdits

⛔ N'écris aucun fichier du dépôt ; aucune commande qui écrit dans le dépôt ou dans une base : `scripts/*` (dont
`scripts/prepare-release.sh`, `scripts/test-fast.sh`), `make`, `latexmk`, `git commit`/`add`/`stash`/`reset`/
`checkout`/`switch`, `sqlx migrate`, `cargo test`/`nextest`, `npm run`, `npx`, `gh issue create`/`comment`/`edit`.
Autorisés : lecture, `grep`, `git log`/`show`/`diff`, `gh issue view`, `cargo check`, `pdftotext` et expériences
dans le scratchpad.
