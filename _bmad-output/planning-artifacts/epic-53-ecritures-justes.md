# Epic 53 — Écritures justes : doublons et comptes du moment

*Kickoff du 2026-11-21 (décision du Project Lead du 2026-11-21, triage P1 des issues ouvertes : tout P1 passe en priorité dans
la release suivante).* Branche `epic/53-ecritures-justes`, partie de `main` au lancement de sa première story (mardi
2026-11-24). Release visée : **vendredi 2026-11-27 si l'epic est clos à la coupure, sinon une release de milieu de semaine
dès sa clôture**. L'epic n'entre dans `main` que terminé.

## Objet

Six défauts P1 qui faussent les livres : une écriture passée deux fois après une réponse perdue, une TVA contre-passée sur
un autre compte que celui de la vente, une ouverture passée sur un compte de résultat, et un compte bancaire dont le lien
au grand livre change sans qu'on le dise.

## Stories

| Story | Issues | Objet |
|---|---|---|
| 53-1 — Une écriture ne passe pas deux fois | #1355, #1365 | après une réponse perdue, un second envoi ne crée ni une seconde écriture (#1355) ni une seconde paire de conversion (#1365). **Même philosophie que la 48-12a** (décision du Project Lead du 2026-11-21) : le serveur refuse le doublon, l'écran fige l'issue inconnue et fait vérifier avant de reprendre. La fiche juge s'il faut la découper en serveur et écran (règle de splitting). |
| 53-2 — La TVA revient sur le compte de la vente | #525, #1198 | l'avoir (#525) et la correction de TVA d'un escompte ou d'une perte (#1198) contre-passent la TVA sur le compte de TVA due **de la vente**, non sur celui des réglages du moment. Probablement une migration (mémoriser le compte sur la facture, ou le relire de l'écriture de vente) : la fiche le dit ; une migration fait monter `b` et sort de préférence un vendredi. |
| 53-3 — L'ouverture ne se passe pas sur un compte de résultat | #513 | le contrôle des types de comptes du bilan d'ouverture se fait dans la transaction qui écrit, sous verrou. |
| 53-4 — L'assistant ne réécrit pas le compte bancaire | #1004 | renvoyer l'étape du compte bancaire ne vide plus sa description et ne remplace plus son compte lié sans le dire : l'écran repart du compte existant. |
| 53-solde — Story de solde | plafond fixé au lancement | dette des revues de l'epic seulement. |

Cinq stories, story de solde comprise : la borne de six tient. Si la 53-1 se découpe en deux, elle en fait six.

## Fichiers et modules (probables, confirmés par les fiches)

- 53-1 : `kesh-api/src/routes/journal_entries.rs`, conversion de devises ; `kesh-db/src/repositories/journal_entries.rs` ;
  `frontend/src/lib/features/journal-entries/` (`JournalEntryForm.svelte`, `ConversionAssistant.svelte`). Motif de
  référence : 48-12a et 48-12b (Epic 50).
- 53-2 : `kesh-db/src/repositories/credit_notes.rs`, `invoice_settlements_write.rs` ; migration éventuelle.
- 53-3 : `kesh-api/src/routes/opening_balances.rs`, `kesh-db/src/repositories/journal_entries.rs`.
- 53-4 : `frontend/src/routes/onboarding/+page.svelte`, `kesh-api/src/routes/onboarding.rs`.

Recoupements : `journal_entries.rs` avec l'Epic 51 (48-2) et l'Epic 47 d'un contributeur ; `onboarding.rs` avec l'Epic 50
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

Plafond : **trois issues** — proposition de l'orchestrateur au lancement, à confirmer par le Project Lead.

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
