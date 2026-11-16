# Epic 35 — choix autonomes de la Story 35-6 (clé `e356`)

Registre de l'agent de développement (Opus 5.5, worktree `kesh-e356`), 2026-11-12/13. Règle d'union C-e350-8 : ce
fichier est propre à la story ; les choix de la fiche sont au registre `e35-choix-autonomes-e350.md` (C-e350-19, -30 à -35).

### C-e356-1 — `serverAccepts` porte la valeur épinglée, non un booléen

- **Contexte** : AC1 nomme le champ `serverAccepts?` sans en fixer le type ; AC8 (e) exige d'épingler la valeur lue par
  le serveur (`1e3` → 1000, `5e-1` → 0,5, la ligne E → 0).
- **Retenu** : `serverAccepts` est la valeur, en chaîne décimale (`"1000"`, `"0.5"`, `"0"`), lue d'une seule source par le
  test de parité. Pour `1,5e2`, la valeur `"150"` est celle que Kesh écrivait aujourd'hui (l'écran changeait la virgule en
  point) ; la ligne sort de l'alphabet du serveur et n'est pas exercée par (e), comme la fiche le dit.
- **Écartées** : un booléen et les valeurs en dur dans le test Rust (deux sources de la même table).
- **Réversible** : oui.

### C-e356-2 — un seul refus pour « vide » et « négatif », et la création qui n'envoie rien sur un vide

- **Contexte** : D6 envoie `negative` sur `bank-accounts-opening-error-amount-required`, que le compte existant emploie
  déjà pour un vide.
- **Retenu** : une fonction `openingAmountRefusal(kind)` dans la page rend le message des quatre issues refusées (un
  site `i18nMsg` par clé, aucun doublon : `sitesTotal` +1, la seule clé neuve). À la création, un vide n'est pas un refus
  (aucun solde envoyé, AC2) ; la fonction n'y est appelée que hors `empty`.
- **Écartée** : un message propre au négatif (une clé de plus, que la fiche ne demande pas).
- **Réversible** : oui.

### C-e356-3 — traductions des deux messages

- **Contexte** : D6 donne le fr-CH ; les trois autres langues sont à écrire (AC5).
- **Retenu** : de-CH, it-CH, en-CH traduits sur le fr-CH, en gardant `1'234`, `1.2340` et `1'234.50` tels quels (la saisie
  est la même dans toutes les langues).
- **Réversible** : oui (à relire par un locuteur à l'occasion de #1050/#1051, qui réemploieront probablement ces textes).

### C-e356-4 — manuel : une phrase sur le montant négatif

- **Contexte** : AC6 décrit le contenu du paragraphe `:2456` ; le refus d'un négatif (`-5`) y était absent.
- **Retenu** : une phrase, « Un montant négatif est refusé : saisissez le montant positif et choisissez le sens. »,
  qui décrit le refus existant (inchangé) au même endroit que les nouveaux refus ; et « une décimale non nulle au-delà de la
  quatrième (`1.23456`) » pour la famille des plus de quatre décimales, qui couvre la famille E sans la nommer (E n'est dite
  qu'au CHANGELOG, D4).
- **Réversible** : oui.

### C-e356-5 — PDF du manuel d'administration et de la brochure non commités

- **Contexte** : `make fr` régénère les trois PDF ; seuls `user-manual.tex` et son PDF changent de contenu.
- **Retenu** : les deux autres PDF restaurés (`git checkout --`), pour ne pas ajouter de conflit binaire à l'union des
  stories parallèles (C-e350-8 : « le PDF se régénère après l'union »).
- **Réversible** : oui (une régénération à l'intégration).

### C-e356-6 — la fixture porte une clé `_doc`

- **Contexte** : la fixture est lue par deux tests de deux langages ; un JSON n'a pas de commentaire.
- **Retenu** : une clé `_doc` en tête, ignorée des deux lecteurs, qui dit d'où vient la table et le sens de chaque champ ;
  les cas sous `cases`.
- **Réversible** : oui.
