# Epic 36 — registre des choix autonomes de la story 36-9 (clé `36-9`)

Registre propre à la story 36-9 (#1044), **versé par union**. Chaque choix : contexte, option retenue, options écartées,
réversibilité. Code lu à `0bf3c3b4b`. Fiche : `36-9-solde-reste-type-compte.md`. Le registre du kickoff
(`e36-choix-autonomes-e360.md`) n'a pas d'étape 0 pour la phase 2 ; C-e360-12 nomme la story. Les décisions de la 35-3
(D2 à D5, `35-3-comptes-designes-type.md`) sont réemployées, non rediscutées.

---

### C-36-9-1 — Le type se contrôle dans l'accesseur du compte de TVA due (fiche, D1)

- **Contexte** : #1044 demande le refus de la 35-3 au solde du reste. Deux lieux possibles : l'accesseur
  `vat_payable_account_for_write` (`company_invoice_settings.rs:669-692`, `FOR UPDATE`), ou la garde à l'usage de la
  validation (`lock_designated_accounts_in_tx` + `check_written`, verrou partagé).
- **Retenu** : l'accesseur lit le type et l'imputabilité dans sa requête verrouillante et rend
  `DesignatedAccountsWrongType` par `DesignatedRole::VatPayable.{expected_type, setting}` et
  `WrongTypeDesignatedAccounts::new` — la constante, la variante, le code et la réponse HTTP de la 35-3.
- **Écartée** : la garde de la validation — verrou partagé à la place de `FOR UPDATE` (ordre des verrous du flux à
  refaire), et un compte archivé passerait de `CONFIGURATION_REQUIRED` à `INACTIVE_OR_INVALID_ACCOUNTS`, un second
  changement de contrat hors de l'objet de #1044.
- **Réversible** : oui ; aucun effet sur les données.

### C-36-9-2 — Le type avant la comparaison à la créance (fiche, D3)

- **Contexte** : le solde compare la TVA due à la créance de la vente (`refuse_if_claim_account`,
  `invoice_settlements_write.rs:686`) juste après l'avoir lue. Une TVA due désignée sur la créance non retypée (un actif)
  est fautive des deux façons.
- **Retenu** : le type d'abord, puisqu'il vit dans l'accesseur — l'ordre déjà suivi par la nature et l'arrondi
  (`usable_designated_account` juge leur type avant leur comparaison). La comparaison reste nécessaire pour une créance
  retypée en passif puis désignée TVA due (test existant `invoice_write_off.rs:1439-1468`).
- **Écartée** : la comparaison d'abord — sortir le contrôle de type de l'accesseur et l'écrire au site d'appel, une
  seconde forme du même contrôle.
- **Réversible** : oui.

### C-36-9-3 — Retypé et non imputable : le refus de type (fiche, D2)

- **Contexte** : la requête de l'accesseur filtre `active = TRUE AND postable = TRUE` et rend
  `CONFIGURATION_REQUIRED` pour tout le reste (C28). La 35-3 (D3) fait primer le type sur le non imputable.
- **Retenu** : `postable` sort du `WHERE` et se juge après le type ; absent ou archivé, puis type, puis non imputable.
  Le refus du non imputable reste `CONFIGURATION_REQUIRED`.
- **Écartée** : `postable` laissé dans le `WHERE` — une ligne de moins, mais un ordre différent de la validation pour le
  même compte.
- **Réversible** : oui.

### C-36-9-4 — La phrase d'action du refus devient neutre (fiche, D4)

- **Contexte** : `error-designated-account-wrong-type-action` finit par « puis la validation pourra être relancée telle
  quelle » ; la réponse HTTP est construite par un bras générique qui ne connaît pas la route. Au solde, « la validation »
  est faux.
- **Retenu** : « … puis l’opération pourra être relancée telle quelle. » dans les quatre langues, pour les trois routes ;
  le changement de texte est écrit à la note de version.
- **Écartées** : un contexte dans la variante `DesignatedAccountsWrongType` et une seconde clé (un champ neuf dans
  `kesh-db`, tous les constructeurs touchés, pour un mot) ; la phrase gardée telle quelle (fausse au solde).
- **Réversible** : oui ; texte seulement, ni code ni `details` ne changent.

### C-36-9-5 — La story ouvre la section *Passer à la 0.18.0* du manuel d'administration (étape 0, V1-5)

- **Contexte** : le changement de contrat de `POST /invoices/{id}/write-off` relève du paragraphe « Pour les
  intégrations » de la section de mise à jour de la release (patron de la 0.17.7, `admin-manual.tex:2563`) ; à
  `0bf3c3b4b`, la section *Passer à la 0.18.0* n'existe pas (`:2538` est *Passer à la 0.17.8*). Le prompt d'appel dit
  que ce paragraphe s'écrit dans la story.
- **Retenu** : la story ouvre `\subsubsection{Passer à la 0.18.0}\label{sec:maj-0-18-0}` avant celle de la 0.17.8, avec
  ce seul paragraphe ; un commentaire LaTeX dit que l'introduction (sauvegarde, aucune migration, retour) revient à
  l'intégration de la release, qui unit les paragraphes des autres stories.
- **Écartées** : laisser le paragraphe à l'intégration (contraire au prompt) ; l'écrire dans la section de la 0.17.8
  (fausse : la 0.17.8 est publiée).
- **Réversible** : oui ; texte seulement. Conflit probable avec une autre story qui ouvrirait la même section : il se
  résout à l'union, en gardant les deux paragraphes.

### C-36-9-6 — Le Vitest qui simule l'ancienne phrase est aligné (étape 0, V1-2)

- **Contexte** : `invoice-validate-page.test.ts:188` simule le message du serveur avec « … puis la validation pourra
  être relancée telle quelle. » ; il resterait vert sans retouche (mock).
- **Retenu** : l'aligner sur la phrase neutre, pour que le simulacre dise ce que le serveur rend ; `npm run test:unit`
  entre dans les gates.
- **Écartée** : le laisser (vert, mais un simulacre d'un texte que le serveur ne rend plus, et un résidu au grep de
  l'AC8).
- **Réversible** : oui.
