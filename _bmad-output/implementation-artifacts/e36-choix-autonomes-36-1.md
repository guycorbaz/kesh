# Epic 36 — registre des choix autonomes de la story 36-1 (#1095)

Registre propre à la story 36-1, **versé par union**. Fiche : `36-1-entree-compte-produit.md`. Code lu à `9d5a142df`.
Ces choix ne sont pas couverts par C-e360 ; ils reprennent l'étape 0 de la 36-1 (registre `e36-choix-autonomes-e360.md`).

### C-36-1-1 — *Entrée* avec un brouillon affiché résout le brouillon et n'envoie jamais (D1)

- **Contexte** : `AccountAutocomplete.svelte:265-269` ne capte *Entrée* que pour une liste ouverte avec un élément en
  surbrillance ; sinon le `<form>` de la facture ou de l'article part avec l'ancien compte pendant que le champ montre le
  texte tapé. L'orchestrateur demande de trancher pour un texte qui correspond à un compte unique, à plusieurs, ou à aucun.
- **Retenu** : dans le composant ; table d'états E0 à E5 de D1 (fiche) : sans brouillon et sans désignation, rien n'est
  choisi et *Entrée* envoie (le champ montre le compte, ou vide = défaut société) ; liste ouverte avec un élément désigné,
  choix visible, unique ou parmi plusieurs ; liste fermée avec correspondances, la liste se rouvre ; aucune correspondance,
  le brouillon reste et un message le dit (si la liste des comptes est chargée) ; brouillon blanc sans désignation, liste
  ouverte ou fermée, la résolution du `blur` (fonction commune `commitDraft()`) ; composition IME et mode dégradé
  inchangés. Une seconde *Entrée* envoie. *(Complété le 2026-11-14 à la remédiation de la validation P1 : la version
  d'origine laissait « vidé, liste ouverte » sans règle, V1-1, et la désignation relève de C-36-1-5.)*
- **Écartées** : résoudre puis envoyer (la saisie jetée en silence au moment où la page change) ; choisir d'office la
  correspondance unique liste fermée (choix non montré, contraire à D2 de la 35-7) ; corriger dans chaque consommateur.
- **Réversible** : oui (une fonction).

### C-36-1-2 — Un message, clé neuve `common-account-no-match` (D2)

- **Contexte** : au cas « aucune correspondance », rien ne part ; sans signal, l'utilisateur ne sait pas pourquoi.
- **Retenu** : une clé globale `common-`, quatre locales, sous `common-account-invalid`, liée par `aria-describedby`,
  dans une région `role="status"` toujours présente (et non un `<p aria-live>` inséré avec son texte — validation P1,
  V1-4), effacée à la frappe, au choix, à l'effacement et au `blur`.
- **Écartée** : jeter le brouillon sans message (texte remplacé sans explication, faute de frappe à retaper).
- **Réversible** : oui.

### C-36-1-3 — Aucun consommateur modifié ; un test pour chacun des deux atteints (D3)

- **Contexte** : sept consommateurs ; seuls `InvoiceForm` et la fiche article sont dans un `<form>` (contrôle `<form>` sur
  chaque fichier et ses parents, cité dans la fiche).
- **Retenu** : correctif dans le composant ; un test Vitest par consommateur atteint (soumission implicite simulée par
  `requestSubmit` quand l'événement n'est pas empêché) et un E2E par écran ; les cinq stricts couverts par les tests du
  composant. *(Complété à la remédiation P1 : C-36-1-5 change la sélection au clavier des sept ; aucun consommateur n'a de
  logique clavier propre au champ et aucun test existant ne sélectionne un compte au clavier — grep cité en D3.)*
- **Écartée** : un test par consommateur strict (ils ne soumettent rien sur *Entrée*).
- **Réversible** : oui.

### C-36-1-4 — Le manuel ne change pas (D4)

- **Contexte** : le manuel ne décrit aucun geste clavier du champ de compte (`git grep` cité dans la fiche).
- **Retenu** : aucune modification ; le message se lit de lui-même ; comme la 35-7.
- **Écartée** : une phrase à `user-manual.tex:1159` (geste décrit pour un écran sur deux, fichier commun avec 36-3 et 36-5).
- **Réversible** : oui, à la demande de la revue.

### C-36-1-5 — #1165 intégrée : une surbrillance ne vaut choix que si elle a été désignée (D1, principe ii)

- **Contexte** : validation P1, V1-2 (MEDIUM) — sur un champ vide non touché, le focus ouvre la liste avec l'index `0`
  (`AccountAutocomplete.svelte:177`, `:327`, `:226`) : la première *Entrée* choisit le premier compte, la seconde envoie ;
  sur une ligne de facture neuve, une double *Entrée* pose le premier compte de produit au lieu du défaut société. #1165
  ouverte depuis la fiche (I4).
- **Décision de l'orchestrateur** (2026-11-14) : #1165 est intégrée à la 36-1 — la surbrillance ne compte comme un choix que
  si le champ porte une saisie ou si une flèche a été pressée ; *Entrée* sur un champ non touché ou vidé sans désignation ne
  choisit aucun compte. La PR ferme #1095 et #1165.
- **Mise en œuvre retenue** : l'index de surbrillance vaut `-1` sans désignation (pas de surbrillance affichée), `0` après
  une frappe non blanche, et les flèches partent de `-1` vers `0`. *(Complété à la remédiation de la validation P2 :
  invariant « index ≥ 0 ⇔ désigné », tenu par un recalcul sur tout changement de la suite des identifiants de `filtered` ou
  de `value` (bouton ×, valeur du parent, comptes arrivés, liste rétrécie) — `$derived` réassignable recommandé ; la flèche
  haut ouvre la liste sur le dernier élément depuis `-1` (patron du combobox ARIA ; écartée : ouvrir sans désigner, qui
  obligerait à une seconde touche) ; le survol passe à `bg-muted` ; table d'*Entrée* close par construction, tout brouillon
  empêchant l'envoi.)* *(Annoté à la remédiation P3 : `bg-muted` n'existe pas dans le thème — voir C-36-1-6 ; le recalcul
  dépend du **texte** du brouillon, non de son seul blanc ; *Entrée* maintenue (`e.repeat`) empêchée et ignorée.)*
- **Écartées** : laisser #1165 à une story ultérieure (même gestionnaire, mêmes tests et E2E à rouvrir) ; un booléen
  `designated` à côté d'un index resté à `0` (deux états à garder d'accord, surbrillance affichée qui ne vaut pas choix).
- **Effet** : la sélection au clavier des sept consommateurs change (première *flèche bas* = premier compte ; *Entrée* sans
  désignation ne choisit rien) ; couverture en D3.
- **Réversible** : oui, une ligne d'état et ses tests.

### C-36-1-6 — Rendre la désignation visible avec des jetons existants, sans définir `--color-accent`

- **Contexte** : validation P3, V3-1 (HIGH) — `frontend/src/app.css` (`@theme`, `:10-52`) ne définit ni `--color-accent` ni
  `--color-muted` ; `bg-accent` et `hover:bg-accent` (`AccountAutocomplete.svelte:396-397`) ne produisent rien : la
  désignation, critère de ce qu'*Entrée* choisit, est invisible.
- **Décision de l'orchestrateur** (2026-11-14) : la 36-1 ne définit pas `--color-accent` (menus et `select` de toute
  l'application, hors périmètre : #1169) ; le composant emploie des jetons définis, choisis par la fiche.
- **Retenu** : désignation `bg-primary/10 ring-1 ring-inset ring-primary` (`--color-primary`, `app.css:12` ; `/10` déjà
  employé, `accounts/+page.svelte:504`) ; survol `hover:bg-border/60` (`--color-border`, `app.css:33`), gris sans filet.
  Contraste du texte sur la désignation mesuré au développement (`tests/e2e/helpers/contrast.ts`).
  *(Remplacé à la remédiation de la validation P4, V4-1 : ces deux fonds pâles se tenaient à 1,04:1 l'un de l'autre et à
  1,18:1 du fond de la liste ; seul le filet de 1 px distinguait la désignation. **Retenu désormais** : désignation
  `bg-primary text-primary-foreground` (`app.css:12`, `:14` ; ≈ 8,7:1 contre `--color-popover`, texte ≈ 8,7:1), survol
  `hover:bg-border` (`app.css:33`) sur les seules options non désignées (désignation contre survol ≈ 7,3:1) ; constantes
  `CLASSE_DESIGNEE`, `CLASSE_SURVOL` dans `account-autocomplete-classes.ts`, gardées par un test de jetons et de contrastes
  sur le patron de `button-destructive.test.ts` (AC8). Écartés : fond atténué + filet + barre gauche — trois classes à garder
  pour un indicateur que le fond plein donne seul.)*
- **Écartées** : `hover:bg-surface-alt` pour le survol (`#f8fafc` sur le fond blanc de la liste : à peine visible) ; définir
  `--color-accent`/`--color-muted` (portée de toute l'application) ; laisser la désignation invisible avec une issue (le
  principe (ii) de D1 reposerait sur un état que l'écran ne montre pas).
- **Réversible** : oui, deux classes.
