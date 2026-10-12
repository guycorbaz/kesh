# Epic 27 — Recette de la v0.13.1 : annuler un import bancaire

**Kickoff : 2026-10-12.** La v0.13.1 est en fonction sur le NAS depuis le 2026-10-12. Guy y a créé
son profil CSV UBS et importé son relevé ; jugeant l'import incomplet, il a voulu l'annuler pour
réimporter — **et rien ne le permet**. **Cible : v0.13.2.**

⚠️ **Contrepartie BMAD de l'issue de recette** — l'issue tient le détail, cet epic le périmètre.
Il se clôt avec [#632] fermée.

## Le constat qui fonde l'epic

Les imports bancaires se **consultent** seulement (`GET /api/v1/bank-imports`,
`GET /api/v1/bank-imports/{id}`) : aucune route, aucun écran ne supprime un import ni ses
transactions. Le seul recours est une requête SQL sur la base de production, ou la restauration
d'une sauvegarde — que Guy écarte (« sans passer par un restaure de la base de données »).

Le réimport du même fichier est en outre **bloqué** par l'unicité
`uq_bank_imports_company_hash (company_id, file_hash)` : un import fautif empêche de le refaire.

## Décision du Project Lead (2026-10-12)

« J'aimerais que tu ajoutes une fonction pour **supprimer les imports non rapprochés**. »

- L'objet supprimé est l'**import entier**, avec ses transactions.
- La suppression n'est permise que si **aucune** de ses transactions n'est rapprochée.

## La story

### 27-1 — Supprimer un import dont aucune transaction n'est rapprochée ([#632])

Points que la spec doit trancher (en autonomie, au registre `epic-27-choix-autonomes.md`) :

- **la définition de « rapprochée »** : statut `reconciled`, `matched_entry_id` non nul — et le
  cas d'une transaction **rejetée** à la réconciliation (statut `pending`, retirée de la liste) ;
  inventorier **tous** les chemins qui écrivent sur `bank_transactions` et toutes les tables qui
  la référencent, plutôt que d'énumérer les deux colonnes connues ;
- **la garde contre la course** : une acceptation de réconciliation concurrente de la suppression
  (verrou, transaction, `version`) ;
- **le rôle** autorisé (celui qui importe : Comptable+) ;
- **la trace d'audit** : code distinct, libellé dans les quatre locales, contenu (fichier,
  période, nombre de transactions) ;
- **l'écran** : où se trouve l'action, sa confirmation, ce qui se passe après (retour à la liste,
  réimport possible) ;
- **l'export et la sauvegarde** : aucun effet attendu, à vérifier ;
- **le manuel** utilisateur (PDF compris) et le CHANGELOG ;
- **aucune migration attendue** — une migration au diff signalerait une dérive.

PR : `closes #632`.

### Release v0.13.2

Après la story : `scripts/prepare-release.sh`, macros des manuels, CHANGELOG, tag annoté —
**sur accord de Guy** (push, PR, merge et tag ne sont pas autorisés d'office pour cet epic).

[#632]: https://github.com/guycorbaz/kesh/issues/632
