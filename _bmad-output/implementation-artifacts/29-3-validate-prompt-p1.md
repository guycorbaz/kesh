# Prompt — validation P1 de la spec, Story 29-3 (la proposition affichée suit une priorité écrite, et l'écran la dit)

*Versionné le 2026-10-14. Deux lentilles, contexte frais chacune, en lecture seule. Rotation (D6, rétro 25) :
passes complètes Sonnet ↔ Opus ; Haiku réservé à une passe ciblée de fin de boucle. P1 : **Sonnet** pour les deux
lentilles (la fiche a été écrite par Opus).*

Dépôt `/home/gcorbaz/devel/kesh-29-3`, branche `story/29-3` (base `16eeed21`, kickoff de l'Epic 29).
Fiche : `_bmad-output/implementation-artifacts/29-3-meilleure-proposition.md` (commit `9f4c54a7`).
Registre de la story : `_bmad-output/implementation-artifacts/epic-29-choix-autonomes-29-3.md` (C-29-3-1 à 10).
Epic : `_bmad-output/planning-artifacts/epic-29-preparer-cloture.md` § *29-3* et § *Ordre*. Issues : #527 (l'objet),
#529 (la 29-2, sérialisée **avant**), #76 (KF-026, une candidate par ligne), #548 (même famille que C-29-3-7) —
`gh api repos/guycorbaz/kesh/issues/527`, `…/529`, `…/76`, `…/548`.
Règles : `CLAUDE.md` (§ Review Iteration Rule et ses sous-sections — le manuel nommé, les axes déclarés, les sites
NON RÉSOLUS, la propagation post-patch, le recompte des comptes rendus ; § Règle de splitting préventif ; § Pattern
batch ; § Un appariement automatique propose, il ne crée jamais) ; `docs/agents-orchestration.md` ; rétrospective
de l'Epic 28 (`epic-28-retro-2026-10-14.md` § 4 et § 7 : périmètre minimal, aucune dette sans issue, aucun
`closes` dans un commit).

⚠️ La 29-2 touchera `routes/reconciliation.rs`, `ReconciliationProposals.svelte` et la même sous-section du manuel,
et sera mergée avant le développement. Les numéros de ligne de la fiche sont relevés sur `16eeed21`, ce qui est le
bon état pour cette passe ; le § *Zones partagées avec la 29-2* est un axe à part entière.

## Axes prioritaires (communs aux deux lentilles)

1. **La règle retenue (C-29-3-1) est-elle juste ?** #527 propose « triées par score » ou « une priorité écrite et
   dite à l'écran ». La fiche choisit trois rangs (facture reconnue / règle / autres factures) et écarte le tri brut.
   Vérifie l'argument au code : le tableau des totaux possibles sous 0,5 (fiche § *Quelles factures sont
   « faibles »*) est-il exact et **complet** (`matching.rs:62-161`, tolérance `AMOUNT_TOLERANCE_HUNDREDTHS`, `HAVING`
   de `find_unpaid_invoices_for_window`) ? `0,0 + 0,4 + 0,1 == 0,5` en `f64`, dans l'ordre d'évaluation de
   `matching.rs:80` ? Le rang 0 peut-il contenir une facture que l'acceptation refuse **autrement** que par le
   trop-perçu de C-29-3-7 ? Le rang 2 peut-il contenir une facture plus sûre que la règle ? Une règle choisie de
   façon incohérente au regard du `CLAUDE.md` (faux rattachement muet) est au moins **HIGH**.
2. **Périmètre minimal** : chaque AC et chaque tâche est-il nécessaire à #527 ou à la non-régression ? La
   réparation du test muet (C-29-3-6) est-elle **mécanique** ? Le défaut voisin C-29-3-7 est-il bien exclu, et son
   angle mort bien écrit (fiche, manuel) ? Un ajout non nécessaire est au moins **MEDIUM** ; une exclusion qui
   laisse #527 ouvert, **HIGH**. Compte les modules touchés depuis les AC (règle de découpage : plus de cinq).
3. **Inventaire clos des sites** qui choisissent, ordonnent ou lisent la candidate « première » : refais et
   **élargis** les recherches — `candidates[0]`, `candidates.first`, `candidates.push`, `top-1`, `.candidates`
   dans `crates/` et `frontend/src/`, les specs E2E (`frontend/tests/e2e/`), les tests HTTP
   (`reconciliation_e2e.rs`, `reconciliation_rules_e2e.rs`, `open_items_e2e.rs`, `admin_full_import_e2e.rs`), un
   client d'API éventuel (`test_endpoints.rs`, clés d'API). Un site qui dépend de l'ordre et que la fiche ignore est
   au moins **HIGH** ; un test existant que le nouvel ordre ferait rougir et que la fiche ne nomme pas, **MEDIUM**.
4. **Manuel contre code** (obligatoire) : `docs/manual/fr/user-manual.tex` § *Algorithme de matching* et
   *Accepter ou rejeter les propositions* (l. 2025-2060), § *Application des règles* (l. 2114-2117), FAQ
   (l. 2896-2912). La prémisse « la meilleure » de #527 est-elle bien réfutée (`grep -nF meilleure`, `git log -S`) ?
   L'inventaire de l'AC7 (quatre sites) est-il **clos** — cherche toute autre phrase qui décrit l'ordre ou la place
   de la règle, y compris `website/`, `README.md`, `CHANGELOG.md` (entrées futures seulement) ? Ce que l'AC7
   prescrit d'écrire sera-t-il vrai au code que la fiche prescrit ? Contrôle aussi le **PDF**, aplati :
   `pdftotext docs/manual/fr/user-manual.pdf - | tr '\n' ' ' | tr -s ' '`.
5. **Recomptes** : AC (9), tâches (T0-T7), mutations (M1-M9), choix (10), cas de l'AC2 (8), sites du manuel (4),
   numéros de ligne cités — chacun depuis la source.

## Lentilles

- **R — Relecture contre le code au sol (auditeur d'acceptation)** : `.claude/skills/bmad-create-story/checklist.md` ;
  chaque phrase de #527 et du § *29-3* de l'epic couverte par un AC testable, chaque AC par une tâche, chaque AC par
  un test **qui rougirait** sans le correctif (l'AC3 cas 1 doit être rouge au sol ; le cas 2 vert au sol et après) ;
  chaque fait, chemin et numéro de ligne **revérifié** (`sed -n`, `grep -nF`) ; les textes prescrits (quatre `.ftl`
  de l'AC5, manuel, CHANGELOG) confrontés au code prescrit, et les traductions DE/IT/EN fidèles au fr-CH ; les
  gardes existantes qui pourraient changer de sens (`lint-i18n-ownership`, `i18n-libelle-en-dur.test.ts`,
  `i18n-keys.test.ts` et ses compteurs, `textes_coherents.rs`, `jetons-non-definis.test.ts`) — la fiche les
  nomme-t-elle ? L'AC4 (« l'acceptation accepte la candidate affichée ») est-il établi au code, `accept_once` et
  `accept_batch` compris, et le Pattern batch intact ? Le § *Zones partagées avec la 29-2* est-il exact et utile
  pour un rebase ?
- **F — Faisabilité et exactitude (adversaire de la conception)** : `candidate_rank` et `order_candidates` (AC1)
  sont-ils réalisables tels quels (types, emprunts, `sort_by_key` stable sur `&mut [T]`, `match` exhaustif sur
  `CandidateType` sans bras `_`) ? Placer l'appel après l'ajout de la règle suffit-il, et rien ne réordonne
  après ? Les cas de l'AC2 donnent-ils bien les ordres annoncés (recalcule chacun) ? Les deux tests HTTP de l'AC3
  sont-ils **réalisables** avec les aides existantes (`seed_validated_invoice`, `seed_bank_transactions`, création
  de règle par l'API dans `reconciliation_e2e.rs`, compte de produit imputable, exercice 2026, contact du client
  et `contact_score` bidirectionnel, montant 99.99 contre 100.00 sous la tolérance) et donnent-ils exactement les
  scores annoncés (0,1 ; 0,4) ? L'acceptation du cas 2 rend-elle bien un règlement partiel (`classify_payment`) et
  non un trop-perçu ? Le test muet (C-29-3-6) est-il réellement muet ? L'E2E de l'AC6 est-il montable (import de
  relevé synthétique, nettoyage) ? Les mutations M1-M9 sont-elles **tuables** par les tests nommés — M6 en
  particulier —, et en manque-t-il une évidente (la phrase de l'AC5 sans `data-testid`, un rang 0 lu sur
  `score.total`) ? Le choix C-29-3-4 (phrase seulement liste non vide) est-il cohérent avec les états de la page
  (chargement, erreur, vide) ?

## Ce que tu rends

Findings numérotés (`R1-…` ou `F1-…`) avec sévérité **CRITICAL / HIGH / MEDIUM / LOW**, `fichier:ligne`,
**preuve** (commande exécutée et sortie copiée), correction proposée. Toute affirmation de présence ou d'absence =
sortie de `grep -nF` copiée. ⛔ **La liste des axes exercés ET non exercés** — un « 0 finding » sans elle ne
compte pas comme passe. Rapport dans `/home/gcorbaz/devel/kesh-gate-logs/29-3-validate-p1-R.md` (lentille R) ou
`/home/gcorbaz/devel/kesh-gate-logs/29-3-validate-p1-F.md` (lentille F) ; dernier message : le chemin du rapport,
le bilan par sévérité, une ligne par finding MEDIUM ou plus.

## Interdits

⛔ N'écris aucun fichier hors `/home/gcorbaz/devel/kesh-gate-logs/`. Aucune commande qui écrit, compile ou
exécute : `scripts/*` (dont **`scripts/prepare-release.sh`**, `scripts/test-fast.sh`, `scripts/mem-guard.sh`),
`make`, `latexmk`, `xelatex`, **`git checkout` et `git switch` sous aucune forme** (ni `--detach`, ni sur un
fichier), `git commit`/`add`/`stash`/`fetch`/`pull`/`reset`/`rebase`/`tag`/`push`/`worktree`, `cargo`, `npm`,
`npx`, `node`, **tout interpréteur** (`python3`, `python`, `perl`, `ruby`, `bash`/`sh` sur un script, `awk` qui
écrit), `docker`, **`mariadb`/`mysql`** et toute requête SQL, `gh` en écriture, `pkill`/`killall`. Aucune donnée
réelle de Guy ne se lit ni ne se cite. Autorisés : lecture, `grep`, `sed -n`, `git log`/`show`/`diff`/`blame`,
`gh api` en lecture, `pdftotext` vers la sortie standard.
