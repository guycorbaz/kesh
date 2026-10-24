# Prompt — revue de code P1 de la Story 32-3 (la couche de services, le motif), trois lentilles, Sonnet

Tu relis le développement de la **Story 32-3** de Kesh (comptabilité suisse, Rust/Axum, MariaDB) : *la couche de
services — le motif, sur deux modules pilotes (rapports, plan comptable)*. Worktree `/home/gcorbaz/devel/kesh-32-3`,
branche `story/32-3`, partie de la fiche validée `08a6f1af` (7 passes de validation, terminée). Le diff à relire,
**aplati** (un seul diff, pas la suite des commits) : `git -C /home/gcorbaz/devel/kesh-32-3 diff 08a6f1af...HEAD`.
Langue du rapport : français. Modèle : **Sonnet**. Contexte frais : ne t'appuie sur aucune passe antérieure.

Lis d'abord (lentilles E et A ; la lentille B lit le diff avant) : la fiche
`_bmad-output/implementation-artifacts/32-3-services-motif.md` — *Ce que le code dit*, *Le motif*, AC1 à AC15, tâches
T0 à T6, *Dev Notes* (règles R1 à R8, *Hors périmètre*), et le **Dev Agent Record** — et le registre
`_bmad-output/implementation-artifacts/epic-32-choix-autonomes-323.md` (C-32-3-1 à C-32-3-33, plus les entrées du
développement, C-32-3-34 et suivantes). Le Dev Agent Record déclare des gates, des rouges et des mutations : **ne les
prends pas pour argent comptant**, recoupe-les avec les journaux `/home/gcorbaz/devel/kesh-gate-logs/323-*.log` et
`323-mut-*.diff` (lecture seule). La story n'ajoute aucun contrôle visible : pas de capture attendue.

## Interdits absolus

- **Lecture seule.** N'écris, ne modifie, ne crée aucun fichier, hormis ton rapport. **Aucun fichier temporaire**, ni
  dans `kesh-gate-logs`, ni dans le scratchpad, ni ailleurs ; **aucune redirection** (`>`, `>>`, `tee`), pas même vers
  un fichier « jetable » : compare par des commandes enchaînées (pipe), lis les PDF vers la sortie standard seulement.
  Un fichier créé par erreur se **déclare** au rapport ; il ne se supprime pas (supprimer, c'est écrire).
- **Aucun `git checkout` ni `git switch`, sous aucune forme** (ni `git checkout -- <fichier>`, ni `git restore`), aucun
  `git` qui écrit (`commit`, `stash`, `reset`, `tag`, `push`, `rebase`, `worktree`, `add`).
- **N'exécute rien qui écrit, compile ou exécute du code** : ni **`scripts/prepare-release.sh`**, ni
  `scripts/test-fast.sh`, ni `scripts/mem-guard.sh`, ni `scripts/install-hooks.sh`, ni aucun script de
  `kesh-gate-logs` ou du scratchpad, ni `make`, ni `latexmk`, ni `cargo` (aucune sous-commande, `cargo fmt` compris),
  ni `rustc`, ni `npm`/`npx`/`node`, ni `docker`, ni `mysql`/`mariadb`, ni `sqlx`, ni `curl`, ni `python3`, ni
  `sed -i`, ni `perl`. Aucune commande `gh` qui écrit ; `gh issue view 723|729|730|431` est permis.
- Permis : `git diff`, `git show`, `git log`, `git grep`, `git ls-files`, `grep`, `sed -n`, `awk` (sans redirection),
  `cat`, `ls`, `find`, `wc`, `pdftotext <f>.pdf - | tr '\n' ' ' | tr -s ' '`.
- **Toute affirmation d'absence ou de présence** s'appuie sur une sortie **`grep -nF`** (ou `sed -n`) copiée.

## Lentille B — Blind (le diff seul, sans la fiche)

Lis le diff aplati **avant** la fiche, comme un relecteur qui ne connaît pas l'intention.

1. `crates/kesh-api/src/services/rapports.rs` : les sept fonctions d'audit sont-elles **identiques** à celles qu'a
   retirées `crates/kesh-api/src/routes/reports.rs` (action, `entity_type`, `AUDIT_ENTITY_ID_NONE`, clés et valeurs du
   JSON, message et champs du `warn!`), seule la source du pool changeant ? Compare au `git show
   08a6f1af:crates/kesh-api/src/routes/reports.rs`. L'aiguillage `tracer` : chacun des huit chemins (forme × usage)
   appelle-t-il la fonction d'aujourd'hui, avec les mêmes arguments (le `fiscal_year_id` de la demande, le filtre de
   journal, les `account_ids` **parsés**, le `mode.as_str()`) ?
2. Les neuf services : l'**ordre** contrôle d'accès → validation → résolution → génération est-il celui d'aujourd'hui
   (comparer chaque handler retiré : `validate_fiscal_year_id` puis `ReportPeriod::resolve` ; `LedgerPeriod::new` puis
   `parse_ledger_account_ids` ; projet : `project_id <= 0`, `resolve_scope`, puis `fiscalYearId` requis) ? Les deux
   régimes du grand livre (affichage `offset` lu, `limit` à `Some(limit.unwrap_or(MAX_LEDGER_LIMIT))` ; export
   `offset 0`, `limit None`) sont-ils tenus ?
3. `crates/kesh-api/src/routes/reports.rs` : chaque handler mince — l'ordre des erreurs (format invalide **avant** tout,
   puis la demande), le `span` d'export (ouvert avant le service, `byte_size`/`duration_ms` enregistrés **dans**
   `rendre`, avant la trace), le contexte PDF chargé **après** la génération, les noms de fichier (période résolue,
   bornes du grand livre, `as_of`, dates de projet) et les en-têtes : identiques au code retiré ? Le `mod tests`
   inchangé octet pour octet ?
4. `Rapport<T, U>` : peut-on obtenir le contenu **sans** écrire la trace (un accesseur, un `Deref`, un champ public) ?
   tracer un export dont le rendu a échoué ? Les accesseurs `periode()`, `bornes_grand_livre()`, `as_of()`,
   `mode_projet()` rendent `Option` et le handler emploie `attendu(…)` (`AppError::Internal`) : défaut de conception ou
   choix tenable (registre) ?
5. `services/acces.rs` : `exiger` — la comparaison `role >= e.role_api` est-elle la même relation que
   `middleware/rbac.rs` (`current_user.role < min_role → Forbidden`) ? Le canal MCP refuse-t-il `portee_mcp: None` ?
6. `services/poignees.rs` et `lib.rs` : champ privé, `pool()` en `pub(in crate::services)`, aucune conversion ;
   `#![cfg_attr(not(test), forbid(unsafe_code))]` au module **et** à la lib — l'attribut de `lib.rs` change-t-il
   quelque chose pour le binaire (`main.rs`) ou pour un autre module ?
7. `helpers.rs` (`company_for_ids`, message identique ?) et `routes/accounts.rs` (`list_accounts` mince, mutations
   inchangées, imports).
8. `crates/kesh-api/tests/services_motif.rs` : le test peut-il passer **à vide** (table vide, aucune trace lue, un
   `continue` qui saute tout, un `Ok` attendu qui serait aussi celui d'un contrôle absent) ? La `Lecture` de test
   est-elle vraiment en lecture seule (témoin 1792) ? Les gardes lexicales (AC10) : faux vert possible (chemin faux,
   motif qui ne trouve rien, définition de « corps de handler ») ?

## Lentille E — Edge cases

1. Demandes limites au service : `fiscalYearId` négatif ; `periodStart > periodEnd` ; `accountIds` vide, `" , "`, `0`,
   non numérique ; grand livre `from == to` ; projet en mode `fiscal_year` sans `fiscalYearId` (l'erreur sort-elle
   **après** `resolve_scope`, comme aujourd'hui ?) ; projet cumulé sans `end_date` ; sous-projet.
2. Balance âgée : `as_of` calculé **une fois** — le contenu, la trace et le nom de fichier lisent-ils la même valeur
   (y compris au passage de minuit UTC) ? Le format d'export est-il bien toujours `csv` en littéral dans la trace ?
3. Canal MCP : portées vides ; une seule des deux ; les deux pour un export (doit être `Forbidden`) ; l'aide
   `identite_societe` sur une société inexistante (refus avant la lecture).
4. `exporter` : `rendre` qui panique ; `rendre` qui réussit et trace qui échoue (best-effort : la réponse reste 200) ;
   `load_pdf_context` qui échoue (aucune trace, le `Rapport` abandonné).
5. Les **mutations** de l'AC12 (M1, M1′, M1-témoin, M2, M2′, M2″, M3, M3′, M4 à M14, M11-bis, M11 élargissement seul) :
   chaque mutation déclarée tuée l'est-elle par l'assertion que la fiche nomme (journaux `323-mut-*.log`, diffs
   `323-mut-*.diff`, `323-campagne.log`) ? Le diff de la mutation est-il celui que décrit la fiche ? Les mutations
   déclarées équivalentes (M12) ou vertes par construction (M11 élargissement seul) le sont-elles vraiment ? Une
   mutation plausible **non jouée** survivrait-elle (p. ex. `exiger` retiré de `compte_de_resultat`, un `report_type`
   faux dans un service, l'export du grand livre sous `offset` lu) ? Raisonne sur le code, n'exécute rien.
6. `AC9 (a)` : quelles branches des générateurs les fixtures n'atteignent-elles **pas** (une écriture sur la `Lecture`
   y resterait muette) ? Le Dev Agent Record les nomme-t-il ?

## Lentille A — Acceptance (chaque AC contre le dépôt)

Pour chaque AC1 à AC15, rends « tenu », « non tenu » ou « non vérifiable en lecture seule », avec la preuve :

- **AC1** : `grep -rnE "axum|AppState|CurrentUser" crates/kesh-api/src/services | grep -vE '^[^:]+:[0-9]+:[[:space:]]*//'`
  → exactement les deux lignes d'`acces.rs` ; cinq fichiers ; attributs `forbid` ; doc-comments.
- **AC2-AC5** : `exiger` en première instruction de **chaque** fonction de service publique ; exigences du tableau de
  l'AC6 (balance âgée : `InvoicesRead`, export `Comptable`) ; `FormatExport` (dérives, méthodes `pub`, alias
  `ExportFormat`) ; `identite_societe` (`{ Consultation, None }`) ; `lister`.
- **AC6** : les dix-neuf chemins du tableau ; mêmes statuts, corps, en-têtes, actions et `details_json` ; le
  doc-comment de conventions rattaché à `emit_report_audit` ; aucune entrée à `SITES_INDIRECTS`.
- **AC7** : `git diff --diff-filter=MDR --name-only $(git merge-base HEAD origin/main) HEAD -- crates/kesh-api/tests`
  → vide (lecture de `git diff` permise) ; bloc `mod tests` de `reports.rs` identique (compare `git show` des deux
  bornes par `sed -n '/^#\[cfg(test)\]/,$p'` en pipe) ; listes des fichiers et des comptes recopiées au Dev Agent
  Record, contre les journaux.
- **AC8-AC10** : chaque volet présent dans `services_motif.rs`, avec assertion nommée ; témoins positifs ; plancher de
  fichiers ; constante d'exceptions de (v-1) nommée et vide.
- **AC11** : inventaire écrit au doc-comment de `services/mod.rs` et au Dev Agent Record ; `grep -rn "get_or_create"`
  sur les fichiers nommés → rien.
- **AC12** : mutations (cf. lentille E, point 5).
- **AC13** : les chiffres du Dev Agent Record (gate ciblé, gate complet `scripts/test-fast.sh`, quatre gates frontend,
  E2E et liste nominative des échecs) correspondent-ils **aux journaux** `323-*.log` ? Les échecs E2E sont-ils jugés
  fichier par fichier contre `docs/testing.md` § *Les échecs attendus* ? Le gate et l'E2E ont-ils tourné **au dernier
  commit de code** (compare la révision des journaux à `git log --oneline 08a6f1af..HEAD`) ?
- **AC14** : aucun texte du manuel (`docs/manual/`, `.tex` **et** PDF aplati), de `docs/api-external.md`,
  `docs/MULTI-TENANT-SCOPING-PATTERNS.md`, `planning-artifacts/architecture.md`, du README ou du CHANGELOG ne décrit
  faussement ce qui bouge ; doc-comments de module de `reports.rs` et `accounts.rs` renvoient à `services`.
- **AC15** : `grep -rniE "transaction read only|tx_read_only" crates/kesh-api/src` → vide ; phrase de `poignees.rs`.
- **Décomptes** : tests ajoutés recomptés (`git show 08a6f1af:<f>` contre `HEAD`, `grep -cE '#\[(sqlx::|tokio::)?test'`),
  périmètre déclaré.
- **Messages de commit** : `git log 08a6f1af..HEAD --format=%B | grep -niE '\b(close[sd]?|fix(e[sd])?|resolve[sd]?)
  #[0-9]'` doit ne rien rendre ; chaque commit porte `refs #723`.
- **Registre et statut** : clé `32-3-services-motif` de `sprint-status.yaml` à `review` ; registre complété par union
  (C-32-3-34 et suivantes), sans réécrire C-32-3-1 à 33.

## Rapport

Écris ton rapport dans `/home/gcorbaz/devel/kesh-gate-logs/323-review-p1-<B|E|A>.md` (selon ta lentille) :

- bilan en tête : CRITICAL / HIGH / MEDIUM / LOW ;
- findings numérotés `B1…`/`E1…`/`A1…`, chacun avec sévérité, fichier:ligne, **preuve copiée** (sortie `grep -nF`,
  `sed -n` ou `git diff`), correctif proposé ;
- **la liste des axes réellement exercés ET de ceux qui ne l'ont pas été** — un « 0 finding » sans cette liste ne compte
  pas comme passe ;
- rien d'autre n'est écrit nulle part.
