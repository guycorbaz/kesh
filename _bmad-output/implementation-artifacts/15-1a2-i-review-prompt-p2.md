# Prompt — revue de code P2, Story 15-1a2-i (le lettrage des pièces clientes)

*Versionné le 2026-10-09. Trois lentilles (Opus), contexte frais chacune, lecture seule. Passe complète : la remédiation P1 touche deux modules de production.*

Dépôt (worktree) `/home/gcorbaz/devel/kesh-15-1a2-i`, branche `story/15-1a2-i-lettrage-des-pieces-clients`.
**Le diff à revoir** : `git diff a06e1927 7b27bf99` (développement `f84931bb` + remédiation P1 `7b27bf99`, sur `main`
qui porte la 15-1a2-0 fusionnée, `a06e1927`). ⛔ **Axe prioritaire, pour chaque lentille** : la remédiation P1
(`git diff 441c16a7 7b27bf99`) — la sévérité se déplace vers ce qu'on vient d'écrire ; le Change Log « Revue de code
P1 » de la fiche dit ce qui a été corrigé, accepté ou réfuté : **conteste** tout verdict faux (B-4 réfuté, E-2 et E-3
acceptés). Rapports P1 : `/home/gcorbaz/devel/kesh-gate-logs/15-1a2-i-review-p1-{B,E,A}.md` — ne les recopie pas, cherche
ce qu'ils ont manqué. Fiche : `_bmad-output/implementation-artifacts/15-1a2-i-lettrage-des-pieces-clients.md`
— P1, P3, P4, P5, P7, AC1–AC5, AC8–AC10, AC12–AC15, AC18, Tests prévus, Dev Notes, et le **Dev Agent Record**, qui
déclare quatre choix (C-15-1a2-i-1 à 4), douze mutations et des décomptes. Fiche amont : `15-1a2-0-lettrage-fige-avec-la-periode.md`
(le refus du rang 2 bis, la règle des périodes `open_period_rule`). Règles : `CLAUDE.md` du dépôt (dont § « Pattern
batch », § « Migration breaking policy », § « Un appariement automatique propose »). Issue : #518 (la story ne la
ferme pas). Registre des choix : `_bmad-output/implementation-artifacts/epic-15-choix-autonomes.md`.

**Contexte** : cette story rend **vivant** le lettrage d'origine `document` : chaque règlement, solde, avoir ou
rapprochement qui solde une facture client la lettre ; l'annulation d'un règlement (ou du solde, ou le
dé-rapprochement) dissout le groupe **avant** la contre-passation. Le refus du rang 2 bis (15-1a2-0) devient
atteignable. Aucune migration. Frontend non touché.

## Lentilles

- **B — Blind Hunter : bugs, concurrence, SQL (`kesh-db`, `kesh-api`)**, sans présupposer que la fiche a raison.
  `letterings.rs` : `discover_invoice_document` (requêtes `FOR UPDATE`, index forcés et `STRAIGHT_JOIN` —
  C-15-1a2-i-2 —, filtre de société, ancre = première ligne au débit de la vente, avoir `issued` seul, règlement
  annulé exclu), `existing_document_group`, `sync_invoice_in_tx` (étapes 1 à 6 : étape 3 terminale, précédence
  `AccountNotLetterable` > `AbstainedClosedPeriods`, comparaison de l'étape 4, branche défensive 5, `check_held_fiscal_year`
  du mode `System`), `dissolve_invoice_document_group_in_tx`, le refactor `create_group_inner` /
  `dissolve_group_inner` (les primitives publiques sont-elles **strictement** inchangées en comportement ?),
  `audit_details`. Les appels : `invoice_settlements_write.rs` (règlement, solde, annulation — place exacte, exercice
  tenu, rien qui précède les refus du geste), `credit_notes.rs`, `routes/reconciliation.rs` (après (g), erreurs
  per-proposal, `lettering_error_to_failed_proposal`, savepoint et rejeu). **Ordre des verrous et instantané** :
  une lecture ordinaire de la synchronisation fige-t-elle une vue `REPEATABLE READ` avant un verrou pris ensuite
  (dont `lines_in_open_period`, `is_letterable_account`) ? Le cycle nommé *rapprochement ‖ règlement* est-il le
  seul neuf ? Un geste peut-il désormais échouer **à cause du lettrage** (P3 : jamais) — énumère les `Err` que la
  synchronisation peut rendre sur un chemin de geste, et dis pour chacune si un état atteignable la produit.
- **E — Edge Case Hunter : bords et états.** Facture soldée par avoir puis… ; règlement daté la veille de la
  facture ; arrondi à 5 centimes négatif (débit d'arrondi dans la vente) ; `SettlesWithRounding` ; solde de la
  nature `rounding` ; facture à cheval sur deux exercices, exercice N clos ; verrou posé entre deux règlements ;
  compte de créance devenu non lettrable puis redevenu lettrable ; avoir hérité sur un autre compte ; facture
  créditée et réglée héritée ; `paid_at` hérité ; dé-rapprochement d'un lien `Entry` hérité qui pointe la vente
  (reçu B-2, verdict au Dev Agent Record : le contester s'il est faux) ; deux règlements concurrents qui soldent
  ensemble ; restauration d'une sauvegarde ; dévalidation (commentaire `invoices.rs` ≈ « inatteignable », toujours
  vrai ?) ; suppression/modification d'une écriture de règlement lettrée `document` (garde `ENTRY_LETTERED`) ;
  contre-passation directe d'une écriture de règlement lettrée. **Cherche le geste ultérieur qui défait
  l'invariant** d'AC9 (« une ligne `document` est sur une pièce en vigueur ») : un chemin qui retire une ligne
  `invoice_settlements` ou passe un avoir hors `issued` **sans** dissoudre ?
- **A — Acceptance Auditor : conformité à la fiche, documentation, manuel.** Chaque AC satisfait ; chaque test de
  « Tests prévus » existe sous son nom et **prouve** ce qu'il nomme (aurait-il échoué avant le patch ? assertion de
  montage devant chaque assertion négative ?) ; la fixture d'AC5 contient bien les deux exceptions et le cas
  « sous la borne » ; AC8 : les détecteurs voient-ils tous les sites (refais toi-même l'inventaire par `grep -rn
  "INTO invoice_settlements\|DELETE FROM invoice_settlements\|INSERT INTO credit_notes\|invoice_settlements::create_in_tx("
  crates --include=*.rs`) ; décomptes du Dev Agent Record **recomptés depuis la source** (29 tests neufs à `7b27bf99` : `git diff
  a06e1927 7b27bf99 -- crates | grep -cE '^\+\s*#\[(sqlx::test|tokio::test|test)'`). **Documentation (AC12, AC18)**
  par la valeur : `CHANGELOG.md`, `docs/api-external.md` (`:225`, `:293`, `:299`, `:326`, les deux tableaux et les deux
  listes en prose des annulations, la table § 10), `docs/manual/fr/user-manual.tex`, `admin-manual.tex`, `README.md`.
  ⛔ **Le manuel se contrôle sur le PDF aplati** : `pdftotext docs/manual/fr/user-manual.pdf - | tr '\n' ' ' | tr -s '
  '` (et `admin-manual.pdf`) vers `/home/gcorbaz/devel/kesh-gate-logs/` — l'item du motif figure-t-il dans les quatre
  listes, l'exception dans la liste des contre-passations, le § du verrou nomme-t-il les deux causes, l'encadré et la
  note portent-ils l'exception, le glossaire est-il juste ? **Le manuel dit-il vrai du code livré ?** — en
  particulier : une facture soldée **avant** cette version est-elle lettrée (non : le rattrapage est la 15-1a2-ii) —
  la documentation le laisse-t-elle croire ? Les textes du refus (fr-CH, la 15-1a2-0) concordent-ils avec ce que le
  manuel en dit ?

## Ce que tu rends

Ton rapport dans `/home/gcorbaz/devel/kesh-gate-logs/15-1a2-i-review-p2-<B|E|A>.md` (ta lettre). Findings avec
sévérité (CRITICAL/HIGH/MEDIUM/LOW), `fichier:ligne`, **preuve** (commande et sortie, ou code relu), correction
proposée. Pour tout finding affirmant qu'un code est absent ou présent : la sortie d'un `grep -nF` copiée.
⛔ **La liste des axes exercés ET non exercés** — un « 0 finding » sans elle ne compte pas.

## Interdits

⛔ N'écris aucun fichier hors de ton rapport dans `/home/gcorbaz/devel/kesh-gate-logs/` (et le texte aplati des PDF
au même endroit) ; aucune commande qui écrit dans le dépôt ou dans une base : `scripts/*` (dont
`scripts/prepare-release.sh`, `scripts/test-fast.sh`, `scripts/mem-guard.sh`), `make`, `latexmk`, `git commit`/
`add`/`checkout`/`switch`/`stash`/`reset`/`rebase`, `sqlx`, `cargo` (aucune sous-commande), `npm`, `npx`, `docker`,
`gh` en écriture, aucune requête SQL. Autorisés : lecture, `grep`, `sed -n`, `git log`/`show`/`diff`, `gh issue view`,
`pdftotext` vers `/home/gcorbaz/devel/kesh-gate-logs/`.
