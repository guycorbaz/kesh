# Prompt — revue de code P1, Story 15-1c-0 (le groupe de lettrage lu avec ses pièces, et la prévision du délettrage)

*Versionné le 2026-10-10. Trois lentilles (Sonnet), contexte frais chacune, lecture seule.*

Dépôt (worktree) `/home/gcorbaz/devel/kesh-15-1c-0`, branche `story/15-1c-0-groupe-de-lettrage-enrichi`.
**Le diff à revoir** : `git diff a602e1ab 732a8495` (le commit de développement ; `a602e1ab` est `origin/main`, la 15-1b
fusionnée ; `HEAD` ne porte en plus que deux numéros de ligne corrigés dans la fiche et ce prompt). Fiche :
`_bmad-output/implementation-artifacts/15-1c-0-groupe-de-lettrage-enrichi.md` — « Contrats consommés », AC15, AC16, AC18,
Tasks, « Tests de T4 » (9 tests numérotés), Dev Notes, et le **Dev Agent Record**, qui déclare trois choix (C-15-1c-0-1 à 3,
registre `epic-15-choix-autonomes.md`), huit mutations, un inventaire et des décomptes. Fiches sœurs, pour savoir ce qui
**n'appartient pas** à cette story : `15-1c-i-ecran-postes-ouverts.md` (l'écran), `15-1c-ii-lettrage-dans-kesh.md` (le reste
de Kesh, le manuel, l'entrée *Ajouté* du CHANGELOG, `closes #518`). Règles : `CLAUDE.md` du dépôt — § « Exception
`kesh-db` », § « Garde-fou défensif » (Pattern batch), § « Recompter ses propres comptes rendus », § « Le prompt d'une
passe doit NOMMER le manuel ». Issue : #518 (la story ne la ferme pas). Hors périmètre : #602, #606, #607.

**Contexte** : story serveur. Dans `kesh-db/src/repositories/letterings.rs` : `ManualDissolutionBlocker` (+ `code()`),
`DissolutionStep`, `manual_dissolution_step` (pure) ; **la dissolution manuelle réécrite sur l'étape** (`dissolve_group_inner`,
boucle) ; `lines_documents_and_periods` / `DocumentsAndPeriods` (la part « pièce et période », appelée par `open_items` et par
la lecture détaillée) ; `find_group_detail` → `LetteringGroupDetail` / `LetteringDetailLine` (`FIND_GROUP_DETAIL_SQL`,
`DetailLineRow`, `GROUP_DETAIL_ACCOUNT_SQL`, transaction de lecture propre). Dans `letterings/open_items.rs` : la requête B
d'`open_items` passe par la fonction partagée. Dans `kesh-api/src/routes/letterings.rs` : `LetteringDetailResponse` pour le
seul `GET`. `docs/api-external.md`, `CHANGELOG.md` (*Modifié*), `README.md`.

⛔ **Axe de sécurité nommé par la fiche — le changement de forme de la dissolution.** C'est la primitive unique du
retrait de la marque (R3) : toute lentille le relit. Ordre des refus 1, 2, 3 inchangé ? séquence de lectures inchangée
(rien lu pour `document`, possession seulement pour `reversal`, borne seulement après) ? verrous d'exercice toujours pris
**avant** le refus 1 ? `blocker`, `document_id`, `document_label` du refus 2 inchangés (rendus en `details` et suffixe du
message par `kesh-api/src/errors.rs`) ? la boucle peut-elle tourner sans fin, ou sortir par un `Invariant` sur un état
atteignable ? mode `System` intact ? `check_rows_affected` toujours appelé (garde `letterings_lexical.rs`) ?

## Lentilles

- **B — Blind Hunter : bugs, SQL, transactions**, sans présupposer que la fiche a raison. La boucle de la dissolution,
  bras par bras (gardes `if any_owned.is_none()`, bras `etape @ …`) ; `manual_dissolution_step` contre l'ancien code
  (`git show a602e1ab:crates/kesh-db/src/repositories/letterings.rs`, refus 1 à 3) — équivalence **sur toutes les entrées**.
  `find_group_detail` : liaisons des marqueurs, transaction ouverte sur une connexion prêtée et sortie par `?` entre
  `begin` et `rollback` (même patron qu'`open_items` — la connexion revient-elle propre au pool ?), ligne d'un autre compte
  sous la même clé (impossible ? que rendrait-on ?), origine prise sur la première ligne. `lines_documents_and_periods` :
  la vue rend-elle **exactement** ce qu'elle rendait (ordre des lectures, `Invariant` sur exercice inconnu, page vide) ?
  DTO : `#[serde(flatten)]` de la ligne — collision de clés possible ? les clés JSON du `POST` intactes ?
- **E — Edge Case Hunter : bords et états.** Groupe à une seule ligne restante (état hérité), groupe aux lignes d'exercices
  multiples, exercice clos / postérieur clos / borne posée au jour d'une ligne, compte retypé ou rattaché à un compte
  bancaire, ligne rapprochée seulement (transaction bancaire : `document` non nul, `ownedByDocument` faux), écriture à
  deux propriétaires (règlement + transaction), facture fournisseur sans numéro (`documentNumber` nul ; message sans
  suffixe), paire `reversal` possédée **et** toute close (quel refus ?), clé d'une autre société (404 indiscernable).
  **Les tests prouvent-ils** ce qu'ils nomment : rejoue les huit mutations du Dev Agent Record et cherche une **mutation
  plausible qui resterait verte** (ex. : la dissolution lit la borne avant la possession ; `inOpenPeriod` de la lecture
  calculé par une autre règle que celle de la vue ; `accountName` d'un autre compte ; `description` et `journal`
  permutés ; prévision `Allowed` quand la lecture n'a aucune ligne en période ouverte pour un groupe `manual`).
- **A — Acceptance Auditor : conformité, décomptes, documentation.** AC15 point par point (forme JSON exacte, « même source
  que la vue » — les deux pièces partagées, une par crate —, `journal`/`description` dans la même requête que les lignes,
  constante SQL et `struct` propres, `find_group`/`LineRow`/`letterable_account`/`group_account_number` inchangés, compte lu
  par sa propre requête, transaction de lecture, prévision, inchangés du `POST`, du `DELETE`, de l'audit, du 404, des
  rôles) ; AC16 (`api-external.md`, le paragraphe du `GET` — `grep -nF 'GET /api/v1/letterings/{key}' docs/api-external.md`
  — dit-il exactement ce que le code fait ? l'exemple respecte-t-il G4-bis : compte `2000` du plan livré ?) ; AC18
  (CHANGELOG *Modifié*, patron « ⚠️ Changement de contrat … » ; README v0.13.0 — et le choix C-15-1c-0-3 d'y déplacer aussi
  la 15-1b). Les tests 1 à 9 existent-ils et prouvent-ils ce qu'ils nomment (en-têtes des tests neufs de
  `crates/kesh-db/tests/letterings.rs` et `crates/kesh-api/tests/letterings_e2e.rs`, `mod tests` de `letterings.rs`) ; les
  tests existants inventoriés au T0 sont-ils **inchangés** (`git diff a602e1ab 732a8495 -- crates/*/tests` : ajouts seuls ?).
  **Décomptes recomptés** (12 tests : 3 / 6 / 3 ; bornes 4 → 7, 38 → 44, 8 → 11 ; sites du `grep` d'unicité). **Écart
  C-15-1c-0-1** (propositions non passées par la fonction partagée) : justifié, ou défaut ? **Manuel** : la story ne le
  touche pas (la 15-1c-ii le fera) — vérifie qu'aucun texte du manuel (`grep -n -i "lettr" docs/manual/fr/*.tex` et le PDF
  aplati, `pdftotext docs/manual/fr/user-manual.pdf - | tr '\n' ' ' | tr -s ' '` vers `/home/gcorbaz/devel/kesh-gate-logs/`)
  n'est rendu **faux** par ce code (ex. : « le délettrage se fait par l'API », toujours vrai ?).

## Ce que tu rends

Ton rapport dans `/home/gcorbaz/devel/kesh-gate-logs/15-1c-0-review-p1-<B|E|A>.md` (ta lettre). Findings avec sévérité
(CRITICAL/HIGH/MEDIUM/LOW), `fichier:ligne`, **preuve** (commande et sortie, ou code relu), correction proposée. Pour tout
finding affirmant qu'un code est absent ou présent : la sortie d'un `grep -nF` copiée.
⛔ **La liste des axes exercés ET non exercés** — un « 0 finding » sans elle ne compte pas.

## Interdits

⛔ N'écris aucun fichier hors de ton rapport dans `/home/gcorbaz/devel/kesh-gate-logs/` (et le texte aplati des PDF au même
endroit) ; aucune commande qui écrit dans le dépôt ou dans une base : `scripts/*` (dont `scripts/prepare-release.sh`,
`scripts/test-fast.sh`, `scripts/mem-guard.sh`), `make`, `latexmk`, `git commit`/`add`/`checkout`/`switch`/`stash`/`reset`/`rebase`,
`sqlx`, `cargo` (aucune sous-commande), `npm`, `npx`, `docker`, `gh` en écriture, aucune requête SQL. Autorisés : lecture,
`grep`, `sed -n`, `git log`/`show`/`diff`, `gh issue view`, `pdftotext` vers `/home/gcorbaz/devel/kesh-gate-logs/`.
