# Prompt — validation P1 de la fiche 50-solde « Solde de la dette de l'Epic 50 » (#1404, #1492, #1414), passe complète

Tu valides la **fiche** de la story de solde de l'**Epic 50** de Kesh (comptabilité suisse ; Rust/Axum, MariaDB, SvelteKit), release
**0.20.0** (vendredi 2026-11-27, coupure le jeudi 2026-11-26 au soir). Kesh tient les livres réels du Project Lead, qui a déjà émis de
vraies factures : rien de ce qui existe (factures, avoirs, écritures, PDF) ne doit changer. **Aucune migration.** Première passe :
**complète**, **contexte frais**, modèle **Sonnet**. Langue du rapport : français.

Worktree `/home/gcorbaz/devel/kesh-50-solde`, branche `story/50-solde-dette-epic`, partie de `origin/epic/50-corrections` à `48a66ca24`.
Fiche : `_bmad-output/implementation-artifacts/50-solde-dette-epic.md`. Inventaires (lecture seule) :
`/home/gcorbaz/devel/kesh-gate-logs/e50solde/inv-1404.txt`, `inv-1492.txt`, `inv-1414.txt`.

## À lire d'abord

1. `CLAUDE.md` du worktree : § *Priorités des défauts* (story de solde), *Qualité du code* (DRY), *Review Iteration Rule* et ses
   sous-sections (*Inventorier les sites NON RÉSOLUS*, *Le prompt d'une passe doit NOMMER le manuel*), *Règle de splitting préventif*,
   *Versions de l'API*, *Recette visuelle*.
2. `_bmad-output/planning-artifacts/epic-50-corrections.md` (périmètre de l'épique, « dette des revues de l'epic seulement »).
3. Les issues, en lecture seule : `gh issue view 1404 --json number,title,labels,body,comments`, idem **1492**, **1414**, et pour le
   contrôle du périmètre : 1441, 1442, 1443, 1415, 1416, 1453, 1491.
4. Les fiches sources : `48-12a-doublons-fournisseurs-serveur.md` (D1, D5, D6bis, D7) et `48-12b-doublons-fournisseurs-ecran.md` (D2,
   table d'états E1 à E9), même dossier.
5. La fiche en entier.

## Axes (chacun exercé, ou déclaré non exercé avec sa raison)

- **A1 — périmètre** : les trois issues sont-elles de la dette **des revues** des stories de l'Epic 50 (origine relue dans les issues et
  les Change Logs des fiches 48-11, 48-12a, 48-12b) ? Les écartées le sont-elles pour la raison dite (priorité, ancienneté, décision du
  Project Lead) ? Le site ajouté au texte de #1404 (le montage de la page) est-il dans la classe, ou un élargissement à déclarer ?
- **A2 — prémisses relues au code** : chaque `chemin:ligne` cité est-il juste à `48a66ca24` (ligne copiée) ? En particulier :
  `new/+page.svelte:106-182`, `imported-supplier-invoices.api.ts:50-62`, `routes/imported_supplier_invoices.rs:90-103` et `:406-415`,
  `lib.rs:765-790`, `imported_supplier_invoices.rs:29-30`, `:119-131`, `:589-603`, `supplier_invoices.rs:494-502`, `:622-674`,
  `kesh-api/src/errors.rs:4346-4349`, `:5583-5598`, `:6253-6290`, `api-client.ts:321-322`, `:414`, `:420`, `:443-448`,
  `api-health.svelte.ts:1-15`, les quatre `messages.ftl:88`, `DegradedBanner.svelte`, `db-resilience.spec.ts:59-61`,
  `supplier-invoice-duplicate.spec.ts:237`, `:275`, `:517-540`, `new-page.test.ts:120-410`. Les deux « à confirmer à l'étape 0 »
  (case (règle 2, dévalidation) de `details.confirmable` aux tests de l'API ; `audit_route_registry` et un `GET`) : tranche-les si tu
  peux, par lecture.
- **A3 — inventaire par le symptôme, ensemble clos** : rejoue les commandes en tête des trois `inv-*.txt` et cherche ce qu'elles
  **omettent** — lot A : une autre lecture d'un document par sa liste (`.find(` / `.some(` sur un résultat de liste, dans le frontend
  et dans `kesh-api`, MCP compris) ; lot B : une autre décision « levable / confirmable » (frontend compris, tests compris) ; lot C : un
  autre texte qui promet un réessai ou une reprise automatique (quatre catalogues, replis Svelte, messages d'erreur Rust). Chaque site
  est-il corrigé ou écrit comme angle mort avec une raison vérifiable ?
- **A4 — décisions** : chaque D a-t-elle son option écartée ? D-A3 rend-elle **exactement** les états d'avant pour chaque statut du
  document (tableau contre `new-page.test.ts:279-410`) ? Le `404` d'un autre code est-il bien traité ? D-B2 laisse-t-elle une seconde
  table déguisée (la correspondance variante ↔ règle) — et si oui, est-elle tenue par un test ? D-C1 : chaque texte est-il vrai pour un
  `GET` comme pour un `POST` (relire `api-health.svelte.ts` : qui éteint le bandeau, et quand) ? Les traductions disent-elles la même
  chose que le français (registre `Sie` en allemand, `tu` en italien, comme les clés voisines) ?
- **A5 — AC constatables** : chaque AC dit-il un test ou une commande, une valeur attendue, un fichier ? AC-A1 (égalité avec l'élément
  de la liste) et AC-A5 (aucune requête `?status=`) sont-ils tenables tels qu'écrits ?
- **A6 — preuve du rouge** : chaque test neuf rougit-il sans le correctif, ou par une mutation nommée avec son journal ? M-B1 et M-B2
  prouvent-elles vraiment qu'une seule table est lue par les deux crates ? Les filets sont-ils déclarés ?
- **A7 — factures réelles et données** : AC-T1 suffit-il à garantir qu'aucune facture, aucun avoir, aucune écriture, aucun PDF ne
  change, et qu'aucun SQL n'est modifié ? Le lot B change-t-il un comportement observable (audit `supplier_invoice.duplicate_confirmed`
  compris) ?
- **A8 — manuel, PDF compris** : vérifie les manuels contre la fiche **et** contre le code, `.tex` et PDF des quatre langues
  (`ls docs/manual/*/` ; `pdftotext docs/manual/<l>/<f>.pdf - | tr '\n' ' ' | tr -s ' '` puis grep de `imported-supplier-invoices`,
  `confirmable`, `confirmNotDuplicate`, `indisponible`, `réessai`, `unavailable`, `retry`). Une phrase que la story rend fausse, ou une
  route neuve que D-A5 oublierait de décrire (guide `docs/api-external.md` compris) ?
- **A9 — splitting et charge** : le décompte de **sept modules** est-il juste (grain de la 48-12b) ? Le signal et les découpages
  proposés sont-ils corrects au regard du `CLAUDE.md` (dérogation, borne de six stories par épique) ? L'estimation tient-elle avant la
  coupure ?
- **A10 — API et versions** : la route neuve respecte-t-elle § *Versions de l'API* (ajout, sans rupture), le RBAC du dossier d'import
  (Comptable, clés `read` pour une méthode sûre) et l'anti-IDOR ?
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
- chaque finding numéroté `V1-n`, sévérité, lot (A, B, C ou transverse), prémisse **citée au code** (`chemin:ligne`, ligne copiée),
  remède proposé ;
- la liste des **axes exercés** et **non exercés** (A1 à A11), avec ce qui a été lancé pour chacun ; un « 0 finding » sans cette liste
  ne compte pas comme passe ;
- les commandes lancées et leurs sorties utiles.
