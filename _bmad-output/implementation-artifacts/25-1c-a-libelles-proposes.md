# Story 25-1c-a — les 122 libellés français, proposés pour relecture

**Écrit le 2026-09-16, pendant l'attente du merge de la PR #437.** Ce document ne modifie aucune fiche
validée : c'est une **proposition**, que la story recopiera dans les quatre catalogues à son implémentation
(AC 15, tâche T4), et que le Project Lead relit ici avant que 488 traductions n'en découlent.

⛔ **Le français d'abord, et lui seul.** L'allemand, l'italien et l'anglais s'écriront à l'implémentation,
d'après ces choix et le glossaire — les traduire maintenant figerait quatre fois une formulation qui peut
encore changer.

## Ce qui fixe ces libellés

- **Les codes viennent du code**, par extraction positionnelle sur les quatre constructeurs de
  `NewAuditLogEntry` (hors tests) : **92 actions** — 82 écrites en toutes lettres, 10 par une variable ou par
  le helper `build_audit_entry` — et **28 types d'entité**. Recompté le 2026-09-16, conforme à l'AC 18.
- **Les mots viennent du glossaire** (`docs/i18n-glossaire.md`, partie A, 76 termes attestés) : *facture*,
  *facture fournisseur*, *avoir*, *écriture (comptable)*, *exercice (comptable)*, *lot (de paiement)*,
  *modèle (d'e-mail)*, *rappel*, *règlement*, *réconciliation*, *import bancaire*, *archiver / archivé*,
  *créé(e)*, *annulé(e)*, *utilisateur*, *rôle*, *rapport*, *projet analytique*, *taux (TVA)*, *compte
  bancaire*, *écarter / écartée (une pièce)*.
- **La forme** : « *Objet* *participe passé* », majuscule initiale, sans point final — ce que lira une
  colonne de tableau et une cellule de CSV.

## Les 28 types d'entité

| code | libellé | code | libellé |
|---|---|---|---|
| `account` | Compte | `installation` | Installation |
| `api_key` | Clé API | `invoice` | Facture |
| `bank_account` | Compte bancaire | `journal_entry` | Écriture |
| `bank_imports` | Import bancaire | `payment_batch` | Lot de paiement |
| `bank_profiles` | Profil bancaire | `product` | Produit |
| `bank_transaction` | Transaction bancaire | `project` | Projet |
| `company` | Société | `reconciliation_rules` | Règle d'affectation |
| `company_dunning_settings` | Réglages de recouvrement | `report` | Rapport |
| `company_invoice_settings` | Réglages de facturation | `supplier_invoice` | Facture fournisseur |
| `contact` | Contact | `user` | Utilisateur |
| `contact_person` | Personne de contact | `vat_rate` | Taux de TVA |
| `credit_note` | Avoir | `export` | Export |
| `dunning_level` | Niveau de rappel | `fiscal_year` | Exercice comptable |
| `email_template` | Modèle d'e-mail | `imported_supplier_invoice` | Facture importée |

⚠️ **Les trois codes au pluriel** (`bank_imports`, `bank_profiles`, `reconciliation_rules`) sont les codes
réels de la base ; leur libellé est au singulier, puisqu'il désigne **une** entrée.

## Les 92 actions

### Comptabilité

| code | libellé |
|---|---|
| `journal_entry.created` | Écriture créée |
| `journal_entry.reversed` | Écriture contre-passée |
| `journal_entry.deleted` | Écriture supprimée |
| `account.created` | Compte créé |
| `account.updated` | Compte modifié |
| `account.archived` | Compte archivé |
| `account.reactivated` | Compte réactivé |
| `fiscal_year.created` | Exercice comptable créé |
| `fiscal_year.updated` | Exercice comptable modifié |
| `fiscal_year.closed` | Exercice comptable clôturé |
| `fiscal_year.reopened` | Exercice comptable rouvert |
| `books.locked` | Période comptable verrouillée |
| `books.unlocked` | Période comptable déverrouillée |
| `books.restored` | Verrou de période rétabli après restauration |
| `vat_rate.created` | Taux de TVA créé |
| `vat_rate.updated` | Taux de TVA modifié |
| `vat_rate.deactivated` | Taux de TVA désactivé |

### Ventes

| code | libellé |
|---|---|
| `invoice.created` | Facture créée |
| `invoice.updated` | Facture modifiée |
| `invoice.validated` | Facture validée |
| `invoice.cancelled` | Facture annulée |
| `invoice.deleted` | Facture supprimée |
| `invoice.emailed` | Facture envoyée par e-mail |
| `invoice.paid` | Facture payée |
| `invoice.partially_settled` | Facture partiellement réglée |
| `invoice.reminder_sent` | Rappel envoyé |
| `invoice.reminder_cancelled` | Rappel annulé |
| `invoice.dunning_paused` | Rappels suspendus |
| `invoice.dunning_resumed` | Rappels repris |
| `credit_note.created` | Avoir créé |
| `dunning_level.created` | Niveau de rappel créé |
| `dunning_level.updated` | Niveau de rappel modifié |
| `dunning_level.deleted` | Niveau de rappel supprimé |
| `company_dunning_settings.updated` | Réglages de recouvrement modifiés |
| `company_invoice_settings.updated` | Réglages de facturation modifiés |

### Achats

| code | libellé |
|---|---|
| `supplier_invoice.created` | Facture fournisseur créée |
| `supplier_invoice.cancelled` | Facture fournisseur annulée |
| `supplier_invoice.paid` | Facture fournisseur payée |
| `imported_supplier_invoice.created` | Facture importée créée |
| `imported_supplier_invoice.completed` | Facture importée complétée |
| `imported_supplier_invoice.discarded` | Facture importée écartée |
| `imported_supplier_invoice.reactivated` | Facture importée réactivée |
| `payment_batch.generated` | Lot de paiement créé |
| `payment_batch.confirmed` | Lot de paiement confirmé |
| `payment_batch.cancelled` | Lot de paiement annulé |

### Banque et réconciliation

| code | libellé |
|---|---|
| `bank_account.created` | Compte bancaire créé |
| `bank_account.updated` | Compte bancaire modifié |
| `bank_account.archived` | Compte bancaire archivé |
| `bank_import.created` | Import bancaire effectué |
| `bank_profile.created` | Profil bancaire créé |
| `bank_profile.updated` | Profil bancaire modifié |
| `bank_profile.deleted` | Profil bancaire supprimé |
| `reconciliation.accepted` | Réconciliation acceptée |
| `reconciliation.rejected` | Réconciliation rejetée |
| `reconciliation.manual_matched` | Réconciliation manuelle effectuée |
| `reconciliation.split_applied` | Ventilation appliquée |
| `reconciliation_rule.created` | Règle d'affectation créée |
| `reconciliation_rule.updated` | Règle d'affectation modifiée |
| `reconciliation_rule.deleted` | Règle d'affectation supprimée |
| `reconciliation_rule.applied` | Règle d'affectation appliquée |

### Tiers, produits, projets

| code | libellé |
|---|---|
| `contact.created` | Contact créé |
| `contact.updated` | Contact modifié |
| `contact.archived` | Contact archivé |
| `contact_person.created` | Personne de contact créée |
| `contact_person.updated` | Personne de contact modifiée |
| `contact_person.archived` | Personne de contact archivée |
| `product.created` | Produit créé |
| `product.updated` | Produit modifié |
| `product.archived` | Produit archivé |
| `project.created` | Projet créé |
| `project.updated` | Projet modifié |
| `project.archived` | Projet archivé |
| `project.unarchived` | Projet réactivé |

### Administration, sécurité, installation

| code | libellé |
|---|---|
| `user.created` | Utilisateur créé |
| `user.updated` | Utilisateur modifié |
| `user.disabled` | Utilisateur désactivé |
| `user.role_changed` | Rôle modifié |
| `user.password_reset` | Mot de passe réinitialisé |
| `api_key.created` | Clé API créée |
| `api_key.revoked` | Clé API révoquée |
| `auth.password_reset_requested` | Réinitialisation de mot de passe demandée |
| `auth.password_reset_completed` | Réinitialisation de mot de passe effectuée |
| `admin_break_glass_reset` | Réinitialisation d'urgence de l'administrateur |
| `company.updated` | Société modifiée |
| `email_template.updated` | Modèle d'e-mail modifié |
| `email_template.restored_default` | Modèle d'e-mail réinitialisé |
| `installation.ui_mode_changed` | Mode d'utilisation changé |
| `admin.full_export` | Sauvegarde exportée |
| `admin.full_import` | Sauvegarde importée |
| `exports.global` | Export global effectué |
| `report.generated` | Rapport généré |
| `report.exported` | Rapport exporté |

**Ventilation, recomptée depuis les tableaux** : 17 + 18 + 10 + 15 + 13 + 19 = **92**.

## Les deux types d'auteur

| clé | libellé |
|---|---|
| `audit-log-actor-type-user` | Utilisateur |
| `audit-log-actor-type-api-key` | Clé API |

## Ce sur quoi je demande votre avis

Cinq libellés ne se déduisent d'aucun terme attesté. Ma proposition est en premier ; l'autre forme suit. Un
sixième, que le catalogue a tranché entre-temps, est gardé ici pour mémoire.

1. **`books.locked` / `books.unlocked` / `books.restored`** — « Période comptable verrouillée /
   déverrouillée » et « Verrou de période rétabli après restauration ». **Vérifié au code** : ces actions
   viennent de `lock_books` et `unlock_books`, qui écrivent `companies.books_locked_through`
   (`repositories/companies.rs:384,512`) — c'est bien le *verrou de période* du manuel, et non les livres
   entiers. L'autre forme, littérale, serait « Livres verrouillés / déverrouillés ».
2. **`project.unarchived`** — « Projet réactivé », pour dire la même chose que `account.reactivated`.
   L'autre forme, « Projet désarchivé », colle au code mais introduit un verbe que le produit n'emploie
   nulle part.
3. **`installation.ui_mode_changed`** — **tranché, pour mémoire** : « Mode d'utilisation changé ». Le réglage
   vaut `Guided` ou `Expert` (`entities/onboarding.rs:10-13`), et le catalogue français dit déjà
   « Choisissez votre mode d'utilisation » (`onboarding-choose-mode`), « Guidé » et « Expert »
   (`mode-guided-label`, `mode-expert-label`). C'est ce mot-là que reprend le libellé — « mode d'affichage »,
   d'abord écrit ici, n'est le mot de personne.
4. **`admin.full_export` / `admin.full_import`** — « Sauvegarde exportée / importée », le mot que le manuel
   emploie. L'autre forme, littérale, serait « Export complet de l'installation ».
5. **`bank_import.created`** — « Import bancaire effectué » plutôt que « créé » : on n'« crée » pas un
   import, on le fait.
6. **`admin_break_glass_reset`** — « Réinitialisation d'urgence de l'administrateur ». C'est la seule action
   sans point, et la seule dont le libellé ne décrit pas un objet du domaine.

⚠️ **Un mot pour le futur** : chaque code d'action ajouté par une story ultérieure devra recevoir son
libellé dans les quatre langues, sans quoi la garde de l'AC 18 rougira. C'est voulu — elle est là pour ça.
