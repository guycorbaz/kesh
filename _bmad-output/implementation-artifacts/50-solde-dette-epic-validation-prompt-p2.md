# Prompt — validation P2 de la fiche 50-solde « Solde de la dette de l'Epic 50 » (#1492, #1414), passe complète

Tu valides la **fiche** de la story de solde de l'**Epic 50** de Kesh (comptabilité suisse ; Rust/Axum, MariaDB, SvelteKit), release
**0.20.0** (vendredi 2026-11-27, coupure le jeudi 2026-11-26 au soir). Kesh tient les livres réels du Project Lead, qui a déjà émis de
vraies factures : rien de ce qui existe (factures, avoirs, écritures, PDF) ne doit changer. **Aucune migration.** Deuxième passe :
**complète** (non ciblée), **contexte frais**, modèle **Opus** (la P1 était Sonnet). Langue du rapport : français.

La fiche porte deux lots indépendants : **B** (#1492, une seule table de levée d'un refus de doublon, `kesh-db` relue par `kesh-api`)
et **C** (#1414, texte du bandeau « base indisponible »). Le lot A (#1404) est sorti vers la v0.21.0 par décision du Project Lead du
2026-11-24 : ne le valide pas. La P1 (Sonnet, 0/0/1/11, `/home/gcorbaz/devel/kesh-gate-logs/e50solde/validation-p1.md`) a été remédiée ;
l'entrée « validation P1 remédiée » du Change Log dit ce qui a changé. **Ne te fie pas à la remédiation : refais chaque axe.**

Worktree `/home/gcorbaz/devel/kesh-50-solde`, branche `story/50-solde-dette-epic`, partie de `origin/epic/50-corrections` à `48a66ca24`
(vérifie par `git rev-parse origin/epic/50-corrections` ; si elle a bougé, relis les citations à la nouvelle tête et dis-le). Fiche :
`_bmad-output/implementation-artifacts/50-solde-dette-epic.md`. Inventaires (lecture seule) :
`/home/gcorbaz/devel/kesh-gate-logs/e50solde/inv-1492.txt`, `inv-1414.txt`.

## À lire d'abord

1. `CLAUDE.md` du worktree : § *Priorités des défauts* (story de solde), *Qualité du code* (DRY), *Review Iteration Rule* et ses
   sous-sections (*Inventorier les sites NON RÉSOLUS*, *Le prompt d'une passe doit NOMMER le manuel*), *Règle de splitting préventif*,
   *Recette visuelle*.
2. `_bmad-output/planning-artifacts/epic-50-corrections.md`.
3. Les issues, en lecture seule : `gh issue view 1492 --json number,title,labels,body,comments`, idem **1414**, et pour le périmètre :
   1404, 1441, 1442, 1443, 1415, 1416, 1453, 1491, 1509.
4. Les fiches sources : `48-12a-doublons-fournisseurs-serveur.md`, `48-12b-doublons-fournisseurs-ecran.md`,
   `48-11-arrondi-au-centime.md` (Change Log, ligne citée pour #1441-#1443), même dossier.
5. La fiche en entier, puis le rapport P1.

## Axes (chacun exercé, ou déclaré non exercé avec sa raison)

- **A0 — la preuve de V1-1 (priorité de cette passe)** : le tableau « Les mutations du lot B, assertion par assertion » du § *Tests*
  distingue-t-il vraiment « une table » de « deux tables » ? Rejoue-le **au raisonnement**, sur le code de `48a66ca24` et le patch décrit
  par D-B1/D-B2 : pour M-B1 et M-B2, quelles assertions rougissent et lesquelles restent vertes, (a) si `kesh-api` relit la table,
  (b) s'il gardait la sienne ? Les attendus de la fiche sont-ils exacts, ligne et itération comprises
  (`supplier_invoice_duplicate_e2e.rs:226-238`, `kesh-db/tests/supplier_invoice_duplicate.rs:1130-1141`) ? Un test AC-2 écrit comme
  l'exige la fiche (deux assertions séparées, messages nommés) rend-il le journal lisible ? M-B3 est-elle cohérente avec D-B2 (code
  déduit de la règle) ? Reste-t-il une mutation plausible du lot B que rien ne fait rougir ?
- **A1 — périmètre** : dette **de revue** des stories de l'Epic 50 ; sources des écartées (#1441-#1443 : décision de Guy du 2026-11-20,
  ligne du Change Log de la 48-11 — relis-la) ; #1414 « comportement antérieur » et le critère de rétention ; #1509 ; aucune trace du
  lot A hors § *Périmètre*, *Dev Notes*, *Change Log*.
- **A2 — prémisses relues au code** : chaque `chemin:ligne` cité est-il juste (ligne copiée) ? En particulier les citations ajoutées par
  la remédiation : `routes/supplier_invoices.rs:349-353`, `:669-673`, `:749-750`, `:956-957` ; `kesh-db/src/errors.rs:580-581` ;
  `supplier_invoices.rs:449-462` ; `kesh-api/src/errors.rs:6240` ; `i18n.svelte.ts:15` ; `SessionIdleWarning.test.ts:8` ;
  `exchange-rates.api.ts:37` ; `db-resilience.spec.ts:31`, `:61`, `:68`, `:96`, `:117-120` ; `it-CH/messages.ftl:804`, `:815`, `:2501` ;
  `fr-CH/messages.ftl:2632-2633`, `:4018`, `:4026` ; et les anciennes (`supplier_invoices.rs:484-502`, `:622-674`, `:1875-1886` ;
  `kesh-api/src/errors.rs:5578-5598`, `:6247-6290`, `:6587`, `:6605` ; `api-client.ts:321-322`, `:414`, `:420`, `:443-448` ;
  `api-health.svelte.ts:1-15` ; quatre `messages.ftl:88` ; `DegradedBanner.svelte:14-17`).
- **A3 — inventaire par le symptôme, ensemble clos** : rejoue les commandes de `inv-1492.txt` et `inv-1414.txt` (ajouts du 2026-11-24
  compris) et cherche ce qu'elles **omettent encore** : une récitation de la table de levée sous une autre forme (anglais, tests,
  `docs/api-external.md`, frontend), un texte qui promet une reprise automatique (quatre catalogues, replis Svelte, Rust).
- **A4 — décisions** : options écartées ; D-B2 (code déduit de la règle) est-elle compilable et exhaustive, et le message reste-t-il
  identique dans les quatre cases ? D-C1 vraie pour `GET` et `POST` ; le doc-comment réécrit de `api-health.svelte.ts` est-il exact
  (toutes les sources qui allument le bandeau, l'exception `degradedOn503`) ? Traductions fidèles ; registre italien à l'infinitif
  cohérent avec les clés citées ?
- **A5 — AC constatables** : dix AC sans trou ; AC-5 (quatre `grep -cF`) tenable avec l'apostrophe U+2019 ; AC-8 (trois scénarios,
  `/health` à 503 pour la capture) tenable.
- **A6 — preuve du rouge** : chaque test neuf rougit-il sans le correctif ou par une mutation nommée avec son journal ? M-C1/M-C2 et la
  note de l'AC-6 sont-elles justes ?
- **A7 — factures réelles et données** : AC-9 et AC-4 ; audit `supplier_invoice.duplicate_confirmed` inchangé.
- **A8 — manuel, PDF compris** : vérifie les manuels contre la fiche **et** contre le code, `.tex` et PDF des quatre langues
  (`ls docs/manual/*/` ; `pdftotext docs/manual/<l>/<f>.pdf - | tr '\n' ' ' | tr -s ' '`, puis grep de `confirmable`,
  `confirmNotDuplicate`, `indisponible`, `réessai`, `unavailable`, `retry`, `Wiederhol`, `tentativo`) ; et `docs/api-external.md`
  (description de `confirmable`). « Aucun manuel ne change » est-il vrai ?
- **A9 — splitting et charge** : quatre modules au grain de la 48-12b (`routes/supplier_invoices.rs` dans `kesh-api`) ; calendrier
  (démarrage mercredi 2026-11-25 au matin, sortie C puis B) réaliste avec les mutations, le gate complet `kesh-db`, `gate-ci`, l'E2E
  complet et deux passes de revue ?
- **A10 — i18n** : clé, domaine, repli Svelte, garde `i18n-repli-divergent-actif`, `lint-i18n-ownership`, apostrophes.
- **A11 — note de version, Dev Notes, commits** : numéros de release, jamais d'épique ; `refs #N` ; aucun mot-clé de fermeture dans la
  fiche ni dans `git log origin/epic/50-corrections..HEAD --format=%B`. Le Change Log respecte-t-il « recompter n'est pas réécrire
  l'historique » (l'entrée de la réduction n'a pas été réécrite) ?

## Interdits

Lecture seule. N'écris **aucun** fichier du worktree ni du dépôt principal ; aucun `Edit`/`Write` hors de ton rapport. Aucun `cargo`,
`npm`, `npx`, `make`, `docker`, `mariadb`, `sqlx`, aucun `scripts/*` (en particulier ni `scripts/prepare-release.sh`, ni
`scripts/test-fast.sh`, ni `scripts/gate-ci.sh`, ni `scripts/mem-guard.sh`, ni `scripts/recette-montee.sh`). Aucun `git commit`,
`git push`, `git checkout`, `git switch`, `git stash`, `git merge`, `git reset`, `git worktree`. Aucun `gh` qui écrit (`issue
create/edit/comment/close`, `pr …`, `project …`). Rien sous `~/kesh/` (production). `pdftotext` et `python3 -I` en lecture pour
recompter sont admis.

## Rapport

Écris **un seul fichier** : `/home/gcorbaz/devel/kesh-gate-logs/e50solde/validation-p2.md`, avec :

- le **verdict chiffré** : CRITICAL / HIGH / MEDIUM / LOW ;
- chaque finding numéroté `V2-n`, sévérité, lot (B, C ou transverse), prémisse **citée au code** (`chemin:ligne`, ligne copiée),
  remède proposé ; dis si un finding **recycle** un finding de la P1 (même défaut sous une autre forme, ou né de la remédiation) ;
- la liste des **axes exercés** et **non exercés** (A0 à A11), avec ce qui a été lancé pour chacun ; un « 0 finding » sans cette liste
  ne compte pas comme passe ;
- les commandes lancées et leurs sorties utiles.
