# Epic 53 — Écritures justes : doublons et comptes du moment

*Kickoff du 2026-11-21 (décision du Project Lead du 2026-11-21, triage P1 des issues ouvertes : tout P1 passe en priorité dans
la release suivante).* Branche `epic/53-ecritures-justes`, partie de `main` au lancement de sa première story (mardi
2026-11-24). Release visée : **v0.21.0, vendredi 2026-12-04** (décision de Guy du 2026-11-24 : la release de correction `x.y.1` ne porte pas de migration, et la 53-2 en porte probablement une). L'epic n'entre dans `main` que terminé.

## Objet

Six défauts P1 qui faussent les livres : une écriture passée deux fois après une réponse perdue, une TVA contre-passée sur
un autre compte que celui de la vente, une ouverture passée sur un compte de résultat, et un compte bancaire dont le lien
au grand livre change sans qu'on le dise ; et, depuis le 2026-11-27, une annulation de rapprochement dont la contre-passation
tombe dans un autre exercice que son écriture (#1674).

## Stories

| Story | Issues | Objet |
|---|---|---|
| 53-1a — La saisie ne passe pas deux fois | #1355 | **née du découpage de la 53-1 le 2026-11-25** (décision du Project Lead, signal de recyclage en validation P2) : après une réponse perdue, un second envoi d'une saisie manuelle identique est refusé (409 levable par confirmation, auditée) ; l'écran fige l'issue inconnue et fait vérifier avant de reprendre ; pose le motif commun (politique de doublon, erreur, module d'états). Aucun verrou neuf sur `journal_entries` : l'instantané naît après le verrou de l'exercice. |
| 53-1b — La conversion ne passe pas deux fois | #1365 | **née du même découpage** : reprend le motif de la 53-1a pour la conversion de devises ; pré-verrou des exercices juste après la société. Dépend de la 53-1a (fusionnée d'abord). |
| 53-2 — La TVA revient sur le compte de la vente | #525, #1198 (reclassées P2 le 2026-11-21, décision du Project Lead : conformité légale) | l'avoir (#525) et la correction de TVA d'un escompte ou d'une perte (#1198) contre-passent la TVA sur le compte de TVA due **de la vente**, non sur celui des réglages du moment. Probablement une migration (mémoriser le compte sur la facture, ou le relire de l'écriture de vente) : la fiche le dit ; une migration fait monter `b` et sort de préférence un vendredi. |
| 53-3 — L'ouverture ne se passe pas sur un compte de résultat | #513 | le contrôle des types de comptes du bilan d'ouverture se fait dans la transaction qui écrit, sous verrou. |
| ~~53-4 — L'assistant ne réécrit pas le compte bancaire~~ | #1004 | **sortie de l'Epic 53 le 2026-11-25** (décision du Project Lead) : la validation P2 de la fiche a établi au code qu'aucun compte principal ne peut exister avant l'étape 6 (`assert_onboarding_complete`, `bank_accounts.rs:358`) ; le défaut ne touche que la saisie de l'assistant, avant toute écriture comptable : #1004 reclassée P3. Fiche et validations conservées sur `story/53-4-assistant-compte-bancaire`, pour la liste d'attente. |
| 53-6 — Annuler un rapprochement contre-passe dans l'exercice de l'écriture | #1674 | **ajoutée le 2026-11-27** (P1, CLO-2, correctif décidé par le Project Lead le 2026-11-26) : la contre-passation d'une annulation de rapprochement est datée de la date de l'écriture, dans son exercice ouvert (rouvert compris) ; exercice clos : refus ; période verrouillée : refus annoncé (proposé, Q2) ; cinq bras (écriture seule, règlement client, règlement fournisseur, groupes client et fournisseur, Wise groupé) ; détection en lecture des contre-passations déjà tombées dans un autre exercice, correction par l'utilisateur (manuel). Aucune migration. Fiche `53-6-annulation-rapprochement-exercice.md`, écrite contre `main` (v0.20.0) : la branche reprend `main` avant le développement. Questions Q1 à Q6 pour Guy. |
| 53-solde — Story de solde | plafond fixé au lancement | dette des revues de l'epic seulement. |

Six stories, story de solde comprise (53-1a, 53-1b, 53-2, 53-3, 53-6, solde 53-5), depuis la sortie de la 53-4, le découpage de la 53-1 (2026-11-25) et l'ajout de la 53-6 (2026-11-27) : la borne de six est atteinte, aucune story ne s'ajoute plus à l'epic.

## Fichiers et modules (probables, confirmés par les fiches)

- 53-1 : `kesh-api/src/routes/journal_entries.rs`, conversion de devises ; `kesh-db/src/repositories/journal_entries.rs` ;
  `frontend/src/lib/features/journal-entries/` (`JournalEntryForm.svelte`, `ConversionAssistant.svelte`). Motif de
  référence : 48-12a et 48-12b (Epic 50).
- 53-2 : `kesh-db/src/repositories/credit_notes.rs`, `invoice_settlements_write.rs` ; migration éventuelle.
- 53-3 : `kesh-api/src/routes/opening_balances.rs` ; `kesh-db/src/repositories/journal_entries.rs` (`create_opening_entry`), `accounts.rs` (lecture des types), `fiscal_years.rs` (clé d'erreur) ; tests `kesh-db` et `kesh-api` (fiche 53-3, validation P1, V1-10).
- ~~53-4~~ : sortie de l'epic (2026-11-25).
- 53-6 : `kesh-db/src/repositories/journal_entries.rs` (`ReversalDating`, `reverse_in_tx_inner`), `reconciliation_cancel.rs`, `settlement_cancellation.rs`, `invoice_settlements_write.rs`, `supplier_invoices.rs`, `grouped_settlements.rs`, `errors.rs` ; `kesh-api/src/routes/reconciliation.rs`, `lib.rs`, `errors.rs` ; `kesh-i18n` (quatre locales) ; `frontend/src/lib/features/reconciliation/` (et le montage d'un panneau sur la page des imports) ; quatre modules, pas de découpage (fiche, § *Règle de splitting préventif*).

Recoupements : `journal_entries.rs` entre la 53-1a/53-1b, la 53-3 et la 53-6 (fonctions distinctes ; la seconde fusionnée reprend la première) ; `invoice_settlements_write.rs` entre la 53-2 et la 53-6 (fonctions distinctes) ; `reconciliation_cancel.rs` entre la 53-6 et l'Epic 39 (Wise, déjà dans `main`), avec l'Epic 51 (48-2) et l'Epic 47 d'un contributeur ; `onboarding.rs` avec l'Epic 50
(aucun site commun connu).

## Dépendances

- Dépend de : l'Epic 50 pour le motif de la 48-12a (à lire, non à attendre).
- Bloque : aucune.

## Lancement (2026-11-23)

### Recoupements contrôlés au code

Contrôle du 2026-11-23 : `git diff --name-only main...<branche>` sur les branches ouvertes, restreint aux fichiers déclarés
ci-dessus.

- **Epic 49 (`epic/49-types-de-lignes-facture`, `story/49-1c-…`)** modifie `kesh-db/src/repositories/credit_notes.rs`,
  `invoice_settlements_write.rs` et `kesh-api/src/routes/credit_notes.rs`, et porte une migration : **recoupement réel
  avec la 53-2**. Règle : l'Epic 49 entre le premier dans `main` (release du vendredi 2026-11-27) ; la 53-2 se développe
  la dernière de l'epic, et sa branche reprend `main` dès l'entrée de l'Epic 49, avant sa revue de code. Sa migration se
  date après celle de la 49-1a.
- **Epic 50 et Epic 51** : aucun fichier commun établi avec les stories 53-1, 53-3 et 53-4 sur leur diff propre ; la 53-1
  lit la 48-12a comme motif, sans l'attendre.
- **Epics d'un contributeur (38, 40, 39, 46)** : leurs PR (#1462, #1463) sont invisibles au 2026-11-23 (compte restreint
  par GitHub) ; le recoupement se contrôle quand elles reparaissent. Leurs deux migrations, avec celles des Epics 49 et 53,
  font de la release qui les accueille une release à migrations (`b` monte, sauvegarde complète obligatoire).

### Ordre des stories

53-1 (doublons), 53-3 (ouverture sous verrou) et 53-4 (assistant) d'abord, indépendantes ; 53-2 (TVA, migration) la
dernière, pour la raison ci-dessus ; story de solde à la fin.

### Story de solde

Plafond : **trois issues** (décision du Project Lead du 2026-11-23, sur proposition de l'orchestrateur). S'il y en a plus, le choix suit la priorité (P2, puis P3, puis P4), puis l'ancienneté ; le surplus reste ouvert, sur une release ultérieure.

### Décomptes au lancement

Recomptés le 2026-11-23 par les commandes du `CLAUDE.md`. Au tag v0.19.1, reconstitution depuis `createdAt`/`closedAt`
(labels du jour, non ceux du tag ; issues masquées du compte d'un contributeur non comptées).

| | Tag v0.19.1 (2026-11-21) | Lancement (2026-11-23) | Objectif au tag de la release (non barrière) |
|---|---|---|---|
| Issues ouvertes | 586 | **567** | moins de 567 |
| Bugs (`bug` ou `known-failure`) | 293 | **274** | moins de 274 |
| P1 ouvertes | — | **10** | six de moins (celles de l'epic) |

Les six issues de l'epic (#1355, #1365, #525, #1198, #513, #1004) sont `bug` et `P1`, au milestone « Vendredi 2026-11-27 »,
au champ *Epic 53* du projet GitHub, en *Todo*.
