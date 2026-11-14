# Epic 36 — Avant la première facture, puis la suite par impact

**Kickoff : 2026-11-14**, en autonomie, après la rétrospective de l'Epic 35 et les réponses de Guy du même jour
(`epic-35-retro-2026-11-14.md`, § 10.1). **Deux phases** : la **0.17.8**, corrective et **sans migration**, à publier
**avant le 2026-11-30** ; puis la suite, dans l'ordre fixé par Guy, jusqu'à la release qui clôt l'epic.

*Rédigé par l'agent de kickoff (Opus 5.5), worktree `kesh-e36`, branche `chore/epic-36-kickoff` partie de `main`
(`42d5f368`). Toutes les lignes citées le sont sur ce commit. Registre des choix du kickoff :
`_bmad-output/implementation-artifacts/e36-choix-autonomes-e360.md` (C-e360-1 à C-e360-14).*

## Objet

Guy tient ses livres réels dans Kesh depuis le 2026-10-31 (0.17.7 sur le NAS depuis le 2026-11-14) et émettra sa
**première facture client avec QR-facture avant le 2026-11-30**. La phase 1 corrige ce qui se trouve sur ce chemin ou
sur celui de sa saisie mensuelle, sans migration, pour que la mise à jour du NAS reste réversible sans restauration. La
phase 2 reprend, dans l'ordre que Guy a donné, la fonction qu'il attend (#823), l'échéance du 1er janvier (#1125), la
remarque interne (seule migration prévue), la dette de la 35-8b découpée par domaine, la QR-facture (#944), les pièces
justificatives (#235) et la **story de solde** que la rétrospective de l'Epic 35 a instituée.

## Le critère d'impact

Celui de l'Epic 35, inchangé (`epic-35-corrections-par-impact.md`, § *Le critère d'impact*) :

| Niveau | Ce que le défaut coûte à Guy quand il se produit |
|---|---|
| **I1** | livres faux ou données perdues |
| **I2** | tâche bloquée, ou contournement coûteux |
| **I3** | gêne, avec un contournement simple |
| **I4** | aucun effet visible sur son travail |

**Son usage** (mémoire de l'orchestrateur, réponses de Guy du 2026-11-12 et du 2026-11-14) : saisie mois par mois,
base petite, MariaDB 10.11.11 ; **facture QR avant le 2026-11-30**, au format de numéro par défaut ; **pain.001** et
lots de paiement ; **projets** ; **rappels** ; **catalogue d'articles** ; un **agent qui écrit par une clé d'API REST**
(le MCP est en lecture seule) ; **non assujetti à la TVA** ; pas d'usage sur téléphone. L'impact est la clé de tri ; la
probabilité dans cet usage départage à impact égal. Les étiquettes `P1`–`P4` ne sont pas changées ici.

## Décomptes de départ

**Au tag v0.17.7** (2026-11-14T12:23:07Z), recomptés depuis les dates de création et de fermeture, labels du jour —
commande du § 2.1 de la rétrospective de l'Epic 35, rejouée au kickoff :

```sh
gh issue list --state all --limit 1500 --json number,labels,createdAt,closedAt --jq '
  def open_at(t): [.[] | select(.createdAt <= t and (.closedAt == null or .closedAt > t))];
  open_at("2026-11-14T12:23:07Z") | length'                                    # 403
# bugs : même filtre, puis select([.labels[].name]|any(.=="bug" or .=="known-failure")) → 212
# P1   : même filtre, puis select([.labels[].name]|index("P1"))                          → 13
```

**Au kickoff** (2026-11-14T14:17Z) :

- **415 issues ouvertes** (`gh issue list --state open --limit 600 --json number --jq length`) ;
- **216 bugs** (commande du § *Priorités des défauts* du `CLAUDE.md`, `--limit 600`) ;
- **13 P1** (`gh issue list --state open --label P1 --limit 100`) : les dix de la TVA (#390 à #397, #401, #525, report
  assumé) et #235, #404, #98.

**Écart tag → kickoff : +12 issues, +4 bugs**, aucune fermée : #1146, #1147, #1148 (issues de la rétrospective), et
#1150 à #1158 (notes de Guy du 2026-11-13 versées en issues le 2026-11-14 : #1150 bug P2, #1151 P4, #1152 P3, #1153 bug
P4, #1154 bug P4, #1155 à #1157 P4, #1158 CR P3). Bugs neufs : #1147, #1150, #1153, #1154.

**Objectif** (règle de release, objectif à terme et **non barrière**) : **moins de 403 issues et moins de 212 bugs au
tag v0.17.8**, à la lettre de la règle (le compte du tag précédent). ⚠️ Au kickoff, le compte est **déjà de 415** ; la
phase 1 en ferme au plus cinq (#1095, #1150, #1051, #1131, #931) — six depuis que la 36-1 intègre #1165 (annoté le 2026-11-14, validation P2 de la fiche 36-1). **L'objectif ne sera pas atteint au tag v0.17.8** sauf
à fermer treize issues de plus, ce qui n'est pas le mandat ; l'écart s'écrira avec sa cause, comme l'exige la règle. Le
tag de fin d'epic vise moins que le compte du tag v0.17.8. **On trace toujours**, une issue par problème ; les deux règles
du 2026-11-14 (*Tarir la source, puis solder*) s'appliquent dès cet epic.

## Phase 1 — release 0.17.8, avant le 2026-11-30, sans migration

| Ordre | Story | Issues | Impact | Objet | Fichiers principaux (lus sur `42d5f368`) | Taille estimée |
|---|---|---|---|---|---|---|
| 1 | **36-1** | #1095, #1165 (intégrée à la validation P1 de la fiche : *Entrée* sur un champ non désigné ne choisit plus le premier compte de la liste) | I2 (I1 si la facture part sans relecture) | *Entrée* dans le champ « compte de produit » d'une ligne de facture ou d'un article ne soumet plus le formulaire avec l'ancien compte pendant que le champ montre le texte tapé | `frontend/src/lib/features/journal-entries/AccountAutocomplete.svelte` (`:255-273`, `:283-317`), `frontend/src/lib/components/invoices/InvoiceForm.svelte` (`:697`, `:877-881`), `frontend/src/routes/(app)/products/+page.svelte` (`:622-627`, `:697-703`), leurs tests | petite |
| 2 | **36-2** | #1150 | I2 | Réconciliation, *Éclater* : le bouton s'active quand les lignes sont complètes et équilibrées, ou l'écran dit ce qui manque | `frontend/src/lib/features/reconciliation/TransactionSplitModal.svelte` (`:132-214`, `:303-316`, `:432-452`) ; selon la cause, `AccountAutocomplete.svelte` ; `tests/e2e/reconciliation-split.spec.ts` | petite ; **cause à établir** (C-e360-5) |
| 3 | **36-3** | #1051 | I1 | Écriture manuelle : « 1.234 » n'est plus lu 1,234 franc en silence — le prédicat d'ambiguïté de la 35-6, non sa grammaire | `frontend/src/lib/features/journal-entries/balance.ts` (`:46-71`), `JournalEntryForm.svelte` (`:206-211`, `:417-427`, `:479`), extraction depuis `frontend/src/lib/features/bank-accounts/format.ts` (`:321-328`), `messages.ftl` ×4, manuel utilisateur | petite à moyenne ; **périmètre borné** à l'écriture manuelle et à l'assistant d'achat, l'apostrophe admise au journal (C-e360-6, décision de Guy du 2026-11-14) |
| 4 | **36-4** | #1131 | I3 | L'annulation d'un règlement — par le dé-rapprochement ou par sa propre route — attribue à la clé d'API ce qu'elle fait | `crates/kesh-db/src/repositories/{invoice_settlements_write.rs (:917, :1142), supplier_invoices.rs (:2815, :3003), reconciliation_cancel.rs (:713-737, :831-858)}`, `crates/kesh-api/src/routes/{invoices.rs (:1387-1405), supplier_invoices.rs (:1015-1029)}`, `crates/kesh-api/tests/audit_acteur_garde.rs` (`:294-299`, `:356-361`), ≈ 15 fichiers de tests, `docs/api-external.md:404` | petite à moyenne ; **`kesh-db` : gate complet en boucle** |
| 5 | **36-5** | #931 (KF-056) | I3 (produit), I4 (test) | Le compte rendu d'une restauration ne se perd plus sous un battement de session refusé ; l'E2E intermittent est fermé, **preuve par répétition avant/après** | `frontend/src/lib/app/stores/session-idle.svelte.ts` (`:326-377`), `frontend/src/lib/features/admin-restore/AdminRestorePanel.svelte` (`:142-165`), peut-être `frontend/src/lib/shared/utils/api-client.ts` (`:194-203`) ; `tests/e2e/backup-restore-new-install.spec.ts` (`:140`, `:262`) | petite |

Les constats de l'étape 0 de chaque story, lignes citées, sont au registre (C-e360-5 à C-e360-8, et § *Étape 0* du
registre). Les fiches passent ensuite par `bmad-create-story`, puis la boucle de validation du `CLAUDE.md`.

### Ordre, dépendances et parallélisme

1. **36-1 en tête** (décision de Guy, Q4 de la rétrospective de l'Epic 35) : c'est l'écran de sa première facture.
2. **Deux lignées**, une par voie de base (consignes communes `kesh-gate-logs/prompts/e35-dev-commun.md`, § *Deux voies
   de base*) — C-e360-4 :
   - **Lignée A, voie 2, frontend** : **36-1 → 36-2 → 36-3**. 36-1 et 36-2 peuvent toucher toutes deux
     `AccountAutocomplete.svelte` (si la cause de #1150 est le brouillon jeté au `blur`, hypothèse H1 de C-e360-5) :
     la seconde part de la première fusionnée. 36-3 touche `JournalEntryForm.svelte`, consommateur d'`AccountAutocomplete`
     que 36-1 ne modifie pas d'après l'étape 0 (il n'est dans aucun `<form>`) ; elle peut se développer en parallèle de
     36-2 si 36-1 est fusionnée.
     *Annoté le 2026-11-14 (validation P1 de la fiche 36-2, C-36-2-7 à 12)* : le champ de montant de l'éclatement passe
     au texte lu par `readJournalAmount` de la 36-3 ; l'ordre devient **36-1 → 36-3 → 36-2** pour le correctif (36-3
     développée en parallèle de 36-1, base de 36-2 = 36-1 et 36-3 fusionnées) ; la reproduction R0 de 36-2 peut se jouer
     avant. #1151 entre dans la 36-2 sans condition.
   - **Lignée B, voie 1, backend puis frontend** : **36-4 → 36-5**. 36-4 touche `kesh-db` (gate complet même en boucle,
     `gate-ci` sur la voie 1 de toute façon) ; 36-5 n'a aucun fichier commun avec elle et peut partir dès que la voie
     se libère.
     *Annoté le 2026-11-14 (validation P2 de la fiche 36-5, V2-7)* : un fichier commun est apparu,
     `docs/manual/fr/admin-manual.tex` (36-4 vers `:3212`, 36-5 vers `:2340`) — paragraphes distincts, PDF régénéré après
     l'union.
3. Les fiches des cinq stories peuvent s'écrire et se valider en parallèle dès le kickoff.

**Fichiers communs repérés** (une story qui en découvre un autre le dit dans son rapport avant de l'écrire) :

| Fichier | Stories | Règle |
|---|---|---|
| `AccountAutocomplete.svelte` | 36-1 (sûr), 36-2 (selon la cause) | en série : 36-2 part de 36-1 fusionnée |
| `TransactionSplitModal.svelte` | 36-2 | — |
| `JournalEntryForm.svelte`, `balance.ts` | 36-3 | — |
| `CHANGELOG.md` | toutes | **aucune branche de story ne l'écrit** : chaque fiche porte sa *note de version* ; l'orchestrateur l'assemble à l'intégration par l'outil Edit seul (C-e360-11, action E8) |
| `messages.ftl` ×4 | 36-3 (sûr), 36-1, 36-2, 36-5 (possible) | jamais en fin de fichier : chaque story à côté du bloc de son écran (`journal-entry-*`, `reconciliation-split-*`, `admin-restore-*`, `invoice-form-*` / `products-*`) |
| `frontend/src/lib/shared/i18n-keys.test.ts` (`sitesTotal`) | toute story qui ajoute un `i18nMsg(` | recompté à la source par chaque branche, puis après chaque fusion, sans additionner (C-e350-18) |
| `docs/manual/fr/user-manual.tex` | 36-3, 36-5 (possible), 36-4 (attribution, avec `admin-manual.tex`) | paragraphes distincts ; PDF régénéré après l'union, contrôlé aplati |
| `docs/api-external.md` | 36-4 (`:404`) | — |
| `crates/kesh-api/tests/audit_acteur_garde.rs` | 36-4, puis les lots 36-11 (phase 2) | 36-4 retire ses entrées `AConvertir` ; les lots partent de 36-4 fusionnée |

### Règles de la phase 1

- **Calendrier** (C-e360-2) : si, **au 2026-11-26**, les cinq stories ne sont pas terminées, la 0.17.8 se publie avec
  celles qui le sont ; **36-1 doit y être**. La mise à jour du NAS reste le geste de Guy, sauvegarde d'abord.
- **Règle d'arrêt — migration** (C-e360-3) : **toute** migration, breaking ou non, empêche le retour à la 0.17.7 sans
  restauration (`CLAUDE.md`, § *Migration breaking policy*, P1). Une story de la phase 1 qui en exige une fait
  **s'arrêter l'orchestrateur**, qui attend Guy. L'étape 0 n'en a trouvé aucune : 36-4 réemploie les colonnes
  `actor_type`, `actor_api_key_id` (migration `20260605000002`), comme la 35-8a.
- **Recette de montée** (`scripts/recette-montee.sh`) depuis l'image 0.17.7 publiée, comme à chaque release.

## Phase 2 — après la 0.17.8, dans l'ordre de Guy

| Ordre | Story | Issues | Impact | Objet | Remarques |
|---|---|---|---|---|---|
| 6 | **36-6** | #823 | I2 | Une transaction qui règle plusieurs écritures (lot pain.001 de plusieurs factures, virement entre comptes propres) | fonction attendue par Guy ; **migration possible** (un lien transaction → écriture unique aujourd'hui, `matched_entry`), **acceptée par Guy le 2026-11-14** dans la release de fin d'epic, sauvegarde d'avant la mise à jour exigée : la fiche la dit avant d'écrire ; découpage préventif probable |
| 7 | **36-7** | #1125 (KF-060) | I4 aujourd'hui, **bloquant au 2027-01-01** | Désancrer de 2026 les tests à exercice ou date fixe | **à terminer d'ici le 2026-12-28** (décision de Guy, Q1) ; critère E1 : `scripts/test-fast.sh --ci` et l'E2E complet verts sous `KESH_TEST_INSTANT=2027-01-04T10:00:00Z`, sonde positive jointe |
| 8 | **36-8** | #1158, #268 | I3 | Remarque (note interne) éditable sur chaque écriture, et note interne par facture émise et reçue | **porte une migration** (non breaking attendue) : release à sauvegarde exigée, en tête des notes et au manuel d'administration |
| 9 | **36-9** | #1044 | I1 | Le solde du reste d'un règlement contrôle le type du compte de TVA due, comme la 35-3 à la validation | même patron que la 35-3 (Q6) ; #525 reste dans le report de la TVA |
| 10 | **36-10** | #1147, #1053 | I3 | Les dernières dates en UTC : filtres et export du journal d'audit, date d'exigibilité d'une facture rappelée | une story pour les deux (Q11) ; prolonge les 35-5a à 35-5c |
| 11 | **36-11a** à **36-11d** | #431, #1132 | I3 | L'acteur de la requête partout où un handler authentifié trace — la 35-8b (`superseded`) découpée en quatre lots par domaine | C-e360-9 ; findings V1-1 à V1-7 de `kesh-gate-logs/e35-8b-validation-p1.md` à traiter dans les fiches ; `closes #431` au dernier lot |
| 12 | **36-12** | #944 | I3 | QR-facture : ciseaux, ligne de séparation, place de *Payable by*, largeurs (SIX § 3.7) | C-e350-1 ; touche le rendu de chaque QR-facture : **après** la première facture réelle de Guy |
| 13 | **36-13** | #235 (P1) | I1 (conformité, art. 957a CO) | Pièces justificatives sur les écritures | **à placer avant la clôture de l'exercice 2026** (Q5) ; migration et stockage probables ; sera découpée à sa fiche, voire portée par un epic à elle |
| 14 | **36-15** | #1161 (CR) | I4 | Parler de « compte analytique » plutôt que de « projet analytique » : vocabulaire visible seulement (écrans, quatre langues, manuels), identifiants internes `project*` gardés | ajoutée le 2026-11-14 sur décision de Guy (C-e360-15) ; **sans migration** ni changement de contrat de l'API ; avant la première clôture d'exercice, pour que les rapports portent le bon terme ; une seule story, faite d'un seul geste |
| 15 | **36-16** | #1167 (CR) | I3 | *Changer le compte analytique* d'une facture client ou fournisseur, quel que soit son statut (payée comprise), et d'une écriture manuelle passée : la facture et toutes ses écritures (vente ou achat, règlements, avoirs, solde) réimputées d'un geste, audité, sans toucher montant, compte, date, numéro, lettrage ni rapprochement | ajoutée le 2026-11-14 sur décision de Guy (C-e360-16) ; **après 36-15** (même objet, vocabulaire fixé) ; sans migration attendue ; touche `kesh-db` : gate complet en boucle ; exercice clos permis avec avertissement (même principe que #1158) |
| 16 | **36-14** | dette des stories 36-* | selon les issues | **Story de solde** : la dette ouverte par les stories de l'epic, **plafonnée à huit issues** dans au plus cinq modules (C-e360-10) | en fin d'epic, avant le tag ; règle « Tarir la source, puis solder » |

**Lots de l'acteur** (C-e360-9 ; le rayon exact se chiffre à l'étape 0 de chaque fiche, V1-4) :

- **36-11a — tiers et catalogue** : `contacts`, `products` ;
- **36-11b — facturation client** : `invoices`, `credit_notes`, `invoice_settlements_write` (hors annulation d'un
  règlement, faite par 36-4), lettrages client ;
- **36-11c — fournisseurs et paiements** : `supplier_invoices` (hors annulation d'un règlement), `payment_batches`,
  `bank_profiles`, `reconciliation_rules` ;
- **36-11d — réglages, exercices, société, restauration, démonstration** : `accounts`, `companies`, `fiscal_years`,
  réglages de facturation, raison `RESTAURATION` de la route d'import (V1-1), `kesh-seed/src/pieges.rs` (#1132, V1-2),
  table finale de la garde, `docs/api-external.md:404` et manuels (V1-3) ; **porte `closes #431`**.

**Hors des stories** (outillage, en chore par une PR à elle, avant la 0.17.8 si possible) : **E2** épingler Node en CI
(#1146) ; **E3** runners (#1055). **Consignes** E5 à E9 de la rétrospective (mutations restaurées sans effacer, forme
permise du contrôle de PDF, une commande lourde à la fois par agent, CHANGELOG sans interpréteur ni `sed -i`, chemins
des prompts vérifiés à la ref) : à écrire au prompt commun de l'Epic 36 par l'orchestrateur, avant le premier agent de
développement.

**Hors de l'epic** : **#1097** (nom de la société non modifiable après l'assistant) n'a **pas de clé** : seulement si
Guy le demande (Q4). Les notes de Guy du 2026-11-13 versées en issues (#1151 à #1157) n'ont pas de clé, sauf ce que
36-2 absorbe (C-e360-5) ; elles restent candidates à une epic suivante ou à la story de solde si l'une de nos stories les
touche. Les trois P1 hors TVA : #235 placé (36-13) ; #404 et #98 restent P1, sans clé.

## Règles de cet epic

- **Autonomie** : un choix de comportement visible prend l'option **prudente et réversible**, s'écrit au registre de la
  story (`e36-choix-autonomes-<clé>.md`) et se relève à la rétrospective.
- **Tarir la source** (`CLAUDE.md`, 2026-11-14) : un LOW de revue qui touche la production peut se corriger dans la
  story, la boucle se rouvrant sur une **passe ciblée** de ce seul correctif ; l'issue reste le choix par défaut quand
  le correctif sort du périmètre.
- **L'étiquette `bug`** décrit un comportement faux visible ; un LOW sans comportement faux prend `technical-debt` ou
  `documentation` (issues nouvelles seulement).
- **On trace toujours**, une issue par problème, sans regroupement des LOW.
- **Limites voisines** : une story corrige les limites courtes qu'elle découvre dans sa zone, dans la mesure du
  possible ; le reste s'ouvre en issue.
- **Validation et revue** : boucles du `CLAUDE.md` — Sonnet ↔ Opus pour les passes complètes, Haiku pour la seule passe
  ciblée de fin de boucle, jamais une première passe ; close sur une passe sans rien au-dessus de LOW ; plafond de huit.
- **Splitting préventif** : plus de cinq modules, ou recyclage d'un défaut d'une passe à l'autre → découpage, déclaré.
- **Gates** (consignes de l'Epic 35, maintenues) : gate ciblé entre les passes (complet si `kesh-db` est touché) ;
  `test-fast --ci` et `gate-ci` au passage en revue ; `gate-ci` et **E2E complet à la clôture**, au dernier commit de code.
- **Worktrees** : retirés **à la clôture de la story**, branche gardée (`CLAUDE.md`, amendé le 2026-11-14, action E4).
- **Agents** : consignes communes de l'Epic 35 (`e35-dev-commun.md`) et celles de l'Epic 36 à écrire (E5-E9) ; l'outil
  Edit seul pour le code. **Production** : 0.17.7 sur le NAS, livres réels ; rien de réel dans le dépôt (fixtures
  synthétiques) ; `~/kesh/` ni lu ni écrit.

## Production

- **Phase 1** : aucune migration ; le retour à la 0.17.7 doit rester possible sans restauration. Recette de montée
  depuis l'image 0.17.7 publiée.
- **Phase 2** : **36-8 porte une migration** ; 36-6 et 36-13 en porteront probablement. Toute release qui en porte une
  exige la sauvegarde d'avant la mise à jour (base, `.keshbackup`, dossier des documents), écrite en tête de ses notes et
  au manuel d'administration ; Kesh écrit en outre la sienne dans `backup/pre-upgrade/` (#847). Une migration breaking
  relève `kesh_version_min_required` **et** les versions Cargo dans le même commit (P2-bis).
