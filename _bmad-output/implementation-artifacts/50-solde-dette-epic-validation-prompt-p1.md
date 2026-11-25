# Prompt — validation P1 de la fiche 50-solde « Solde de la dette de l'Epic 50 » (#1492, #1414), passe complète

Tu valides la **fiche** de la story de solde de l'**Epic 50** de Kesh (comptabilité suisse ; Rust/Axum, MariaDB, SvelteKit), release
**0.20.0** (vendredi 2026-11-27, coupure le jeudi 2026-11-26 au soir). Kesh tient les livres réels du Project Lead, qui a déjà émis de
vraies factures : rien de ce qui existe (factures, avoirs, écritures, PDF) ne doit changer. **Aucune migration.** Première passe :
**complète**, **contexte frais**, modèle **Sonnet**. Langue du rapport : français.

La fiche porte deux lots indépendants : **B** (#1492, une seule table de levée d'un refus de doublon, `kesh-db` relue par `kesh-api`)
et **C** (#1414, texte du bandeau « base indisponible »). Un lot A (#1404) a été **sorti vers la v0.21.0 par décision du Project Lead
du 2026-11-24** : ne le valide pas, contrôle seulement que la fiche n'en garde aucune trace hors du § *Périmètre*, des *Dev Notes* et du
*Change Log*.

Worktree `/home/gcorbaz/devel/kesh-50-solde`, branche `story/50-solde-dette-epic`, partie de `origin/epic/50-corrections` à `48a66ca24`.
Fiche : `_bmad-output/implementation-artifacts/50-solde-dette-epic.md`. Inventaires (lecture seule) :
`/home/gcorbaz/devel/kesh-gate-logs/e50solde/inv-1492.txt`, `inv-1414.txt`.

## À lire d'abord

1. `CLAUDE.md` du worktree : § *Priorités des défauts* (story de solde), *Qualité du code* (DRY), *Review Iteration Rule* et ses
   sous-sections (*Inventorier les sites NON RÉSOLUS*, *Le prompt d'une passe doit NOMMER le manuel*), *Règle de splitting préventif*,
   *Recette visuelle*.
2. `_bmad-output/planning-artifacts/epic-50-corrections.md` (périmètre de l'épique, « dette des revues de l'epic seulement »).
3. Les issues, en lecture seule : `gh issue view 1492 --json number,title,labels,body,comments`, idem **1414**, et pour le contrôle du
   périmètre : 1404, 1441, 1442, 1443, 1415, 1416, 1453, 1491.
4. Les fiches sources : `48-12a-doublons-fournisseurs-serveur.md` (D1, D5, D6bis, D7) et `48-12b-doublons-fournisseurs-ecran.md`
   (table d'états, E6), même dossier.
5. La fiche en entier.

## Axes (chacun exercé, ou déclaré non exercé avec sa raison)

- **A1 — périmètre** : les deux issues sont-elles de la dette **des revues** des stories de l'Epic 50 (origine relue dans les issues et
  les Change Logs des fiches 48-12a, 48-12b) ? Les écartées et la sortie de #1404 sont-elles dites avec leur raison ? Reste-t-il dans la
  fiche une décision, un AC, une tâche, une mutation ou un fichier du lot A ?
- **A2 — prémisses relues au code** : chaque `chemin:ligne` cité est-il juste à `48a66ca24` (ligne copiée) ? En particulier :
  `supplier_invoices.rs:449-460`, `:484-502`, `:622-674`, `:944`, `:1152`, `:1875-1886`, `:3838` ; `kesh-db/src/errors.rs:574`,
  `:1892-1912`, `:2606-2607` ; `kesh-api/src/errors.rs:5578-5598`, `:6247-6290`, `:6587`, `:6605` ; `supplier-invoice-form.ts:592-619` ;
  `api-client.ts:321-322`, `:414`, `:420`, `:443-448` ; `api-health.svelte.ts:1-15` ; les quatre `messages.ftl:88` ;
  `DegradedBanner.svelte:14-17` ; `db-resilience.spec.ts:59-61` ; `supplier-invoice-unknown-outcome.spec.ts:161` ;
  `supplier_invoice_duplicate_e2e.rs:238`, `:263`, `:421` ; `kesh-db/tests/supplier_invoice_duplicate.rs:1136`. La prémisse « à
  confirmer à l'étape 0 » (case (règle 2, dévalidation) de `details.confirmable` aux tests de l'API) : tranche-la par lecture.
- **A3 — inventaire par le symptôme, ensemble clos** : rejoue les commandes en tête des deux `inv-*.txt` et cherche ce qu'elles
  **omettent** — lot B : une autre décision « levable / confirmable » d'un doublon (frontend, tests, MCP compris ; une décision écrite
  sans les mots « liftable » ni « confirmable ») ; lot C : un autre texte qui promet un réessai ou une reprise automatique (quatre
  catalogues, replis Svelte, messages d'erreur Rust, `error-network`, `error-timeout`). Chaque site est-il corrigé ou écrit comme angle
  mort avec une raison vérifiable ?
- **A4 — décisions** : chaque D a-t-elle son option écartée ? D-B2 laisse-t-elle une seconde table déguisée (la correspondance variante
  ↔ règle) — et si oui, M-B3 et l'AC-2 la tiennent-ils ? Le `match` de D-B1 reste-t-il exhaustif ? D-C1 : chaque texte est-il vrai pour
  un `GET` comme pour un `POST` (relire `api-health.svelte.ts` et `api-client.ts` : qui allume et qui éteint le bandeau, et quand) ? Les
  traductions disent-elles la même chose que le français, au registre des clés voisines (`Sie`, `tu`) ?
- **A5 — AC constatables** : chaque AC dit-il un test ou une commande, une valeur attendue, un fichier ? Les dix AC sont-ils numérotés
  sans trou et cités juste dans les tests et les tâches ?
- **A6 — preuve du rouge** : chaque test neuf rougit-il sans le correctif, ou par une mutation nommée avec son journal ? M-B1 et M-B2
  prouvent-elles vraiment qu'une seule table est lue par les deux crates ? Les filets (AC-2) sont-ils déclarés ?
- **A7 — factures réelles et données** : AC-9 suffit-il à garantir qu'aucune facture, aucun avoir, aucune écriture, aucun PDF ne
  change, qu'aucun SQL n'est modifié, et que l'audit `supplier_invoice.duplicate_confirmed` s'écrit dans les mêmes cas ?
- **A8 — manuel, PDF compris** : vérifie les manuels contre la fiche **et** contre le code, `.tex` et PDF des quatre langues
  (`ls docs/manual/*/` ; `pdftotext docs/manual/<l>/<f>.pdf - | tr '\n' ' ' | tr -s ' '`, puis grep de `confirmable`,
  `confirmNotDuplicate`, `indisponible`, `réessai`, `unavailable`, `retry`, `Wiederhol`, `tentativo`). La fiche dit qu'aucun manuel ne
  change : est-ce vrai ? `docs/api-external.md` (description de `confirmable`) reste-t-il exact ?
- **A9 — splitting et charge** : le décompte de **quatre modules** est-il juste (grain de la 48-12b) ? L'estimation (une demi-journée)
  tient-elle avant la coupure ?
- **A10 — i18n** : la clé change-t-elle de valeur sans changer de nom ni de domaine (`lint-i18n-ownership`) ? Le repli Svelte et la
  garde `i18n-repli-divergent-actif` sont-ils correctement mobilisés ? Apostrophes typographiques cohérentes avec les clés récentes ?
- **A11 — note de version, Dev Notes, commits** : numéros de release, jamais d'épique ; `refs #N` ; aucun mot-clé de fermeture dans la
  fiche ni dans `git log origin/epic/50-corrections..HEAD --format=%B`.

## Interdits

Lecture seule. N'écris **aucun** fichier du worktree ni du dépôt principal ; aucun `Edit`/`Write` hors de ton rapport. Aucun `cargo`,
`npm`, `npx`, `make`, `docker`, `mariadb`, `sqlx`, aucun `scripts/*` (en particulier ni `scripts/prepare-release.sh`, ni
`scripts/test-fast.sh`, ni `scripts/gate-ci.sh`, ni `scripts/mem-guard.sh`, ni `scripts/recette-montee.sh`). Aucun `git commit`,
`git push`, `git checkout`, `git switch`, `git stash`, `git merge`, `git reset`, `git worktree`. Aucun `gh` qui écrit (`issue
create/edit/comment/close`, `pr …`, `project …`). Rien sous `~/kesh/` (production). `pdftotext` et `python3 -I` en lecture pour
recompter sont admis.

## Rapport

Écris **un seul fichier** : `/home/gcorbaz/devel/kesh-gate-logs/e50solde/validation-p1.md`, avec :

- le **verdict chiffré** : CRITICAL / HIGH / MEDIUM / LOW ;
- chaque finding numéroté `V1-n`, sévérité, lot (B, C ou transverse), prémisse **citée au code** (`chemin:ligne`, ligne copiée),
  remède proposé ;
- la liste des **axes exercés** et **non exercés** (A1 à A11), avec ce qui a été lancé pour chacun ; un « 0 finding » sans cette liste
  ne compte pas comme passe ;
- les commandes lancées et leurs sorties utiles.
