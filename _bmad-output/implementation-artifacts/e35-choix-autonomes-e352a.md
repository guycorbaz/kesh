# Epic 35 — choix autonomes de la story 35-2a (clé `e352a`)

Registre tenu par l'agent de développement de la 35-2a (Opus 5.5, worktree `kesh-e352a`), le 2026-11-12. Mise à jour
par union, jamais par écrasement.

### C-e352a-1 — Les noms des clés et les phrases du refus

- **Contexte** : D3 fixe la forme (une clé par champ et par raison) et laisse les noms libres ; D5 fixe une phrase par
  cause, sans en donner le texte hors des exemples de D6, D7 et D9.
- **Retenu** : `invoice-pdf-error-{charset|empty|too-long}-{company|client}-{name|street|building|postal-code|city}`,
  `invoice-pdf-error-charset-invoice-number`, `invoice-pdf-error-{charset|empty|too-long}-other` pour un nom de champ
  inconnu, et `invoice-pdf-error-{iban-invalid|qr-iban-invalid|internal|amount-invalid|currency-invalid|country-invalid}`,
  `-country-invalid-{company|client}`, `credit-note-pdf-error-contact-missing`. Chaque phrase dit le champ, la partie, le
  caractère ou la limite, puis **où corriger** : *Paramètres → Organisation* pour l'entreprise (l'écran qui porte son nom
  et son adresse), « la fiche du client dans le carnet d'adresses » pour le client (`nav-contacts` fr-CH), *Administration
  → Comptes bancaires* pour l'IBAN, *Administration → Facturation* pour le numéro de facture. « numéro de bâtiment » pour
  `line2`, « rue » pour `line1`, « localité » pour `town` (les mots de la saisie de l'adresse, `address_input.rs`).
- **Écartées** : une phrase sans remède (l'utilisateur sait quoi, pas où) ; « Administration → Entreprise », qui
  n'existe pas.
- **Réversible** : oui, texte seul ; le changer touche le catalogue, la fonction pure et G9 ensemble.

### C-e352a-2 — La cause garde le point de code, non le caractère

- **Contexte** : `QrBillError::InvalidCharset` porte `codepoint: u32`.
- **Retenu** : `CauseRefusPdf::CaractereRefuse { champ, codepoint }` ; `refus_pdf` le convertit par `char::from_u32`, avec
  U+FFFD si la conversion échouait (inatteignable : le point vient d'un `char`), et écrit le point depuis le `u32`, si
  bien qu'il reste juste même dans ce cas.
- **Écartée** : `caractere: char`, qui obligerait `map_qrbill_error` à choisir le repli.
- **Réversible** : oui.

### C-e352a-3 — Où se place le contrôle du pays (D7)

- **Contexte** : D7 demande de contrôler le pays de chaque partie avant `kesh-qrbill`, sans dire où.
- **Retenu** : juste après la construction de l'adresse de chaque partie, une fois le pays résolu (celui de l'adresse
  structurée, sinon la colonne `country`), et après le contrôle de complétude (D6). Un document dont le pays **et** un
  autre champ sont refusés nomme donc d'abord le pays (de l'entreprise, puis du client), là où `kesh-qrbill` aurait nommé
  le nom ou un caractère : un refus à la fois, comme aujourd'hui.
- **Écartée** : contrôler les deux pays avant les adresses — l'ordre des messages s'écarterait davantage de celui de
  `kesh-qrbill` sans rien gagner.
- **Réversible** : oui.

### C-e352a-4 — La garde AC14 retire les modules de test par appariement d'accolades

- **Contexte** : AC14 compte `InvoiceNotPdfReady(` « hors modules `#[cfg(test)]` ». Couper chaque fichier au premier
  `#[cfg(test)]` cacherait du code de production écrit après un module de test (`config.rs` en a plusieurs).
- **Retenu** : `sans_modules_de_test` retire chaque `#[cfg(test)] mod … { … }` en appariant les accolades, sans compter
  celles des chaînes (y compris brutes), des caractères et des commentaires ; un anti-test-muet
  (`sans_modules_de_test_apparie_les_accolades`) le prouve sur un source construit.
- **Écartée** : la coupe au premier `#[cfg(test)]`.
- **Réversible** : oui.

### C-e352a-5 — La preuve du rouge sur des binaires du code de départ

- **Contexte** : les tests neufs ont été écrits avant le code ; la compilation du workspace a fini après les premières
  modifications du catalogue, lu à l'exécution.
- **Retenu** : les binaires de test compilés sur le code de départ (vérifiés : ils portent « Pays invalide » et
  `qrbill-error-qrr-generation`) ont été lancés directement, **catalogues de la 0.17.6 remis en place** le temps du run
  (`git stash push -- crates/kesh-i18n/locales`, puis `pop`). Journal : `kesh-gate-logs/e352a/rouges.log`.
- **Écartée** : relancer la compilation sur un état reconstitué (dix minutes de plus, même preuve).
- **Réversible** : sans objet.

### C-e352a-6 — `last_updated` du `sprint-status.yaml` non touché

- **Contexte** : le workflow demande de mettre à jour `last_updated` ; la 35-3 se développe en parallèle et écrit au même
  fichier, dont l'en-tête est une pile de lignes `# last_updated`.
- **Retenu** : seule la clé `35-2a-…` change ; l'en-tête s'écrira à l'intégration.
- **Réversible** : oui.

### C-e352a-7 — AC6, la QRR : prouvée au niveau de la fonction, non par HTTP

- **Contexte** : `build_qrr` n'échoue pas pour un identifiant de société ou de facture réel, et `InvalidQrr` ne sort pas
  d'une QRR calculée par Kesh : aucun montage HTTP n'atteint la panne interne.
- **Retenu** : la phrase « non corrigeable » est assertée par la table d'AC2 (`PanneInterne`, quatre langues) et par
  `map_qrbill_error_rend_la_cause_de_chaque_variante` (`InvalidQrr`, `InvalidPayload`). Angle mort assumé : le chemin
  `:338` (échec de `build_qrr`) n'a pas de test qui le traverse.
- **Réversible** : oui.
