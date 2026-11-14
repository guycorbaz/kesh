# Epic 36 — registre des choix autonomes de la story 36-1 (#1095)

Registre propre à la story 36-1, **versé par union**. Fiche : `36-1-entree-compte-produit.md`. Code lu à `9d5a142df`.
Ces choix ne sont pas couverts par C-e360 ; ils reprennent l'étape 0 de la 36-1 (registre `e36-choix-autonomes-e360.md`).

### C-36-1-1 — *Entrée* avec un brouillon affiché résout le brouillon et n'envoie jamais (D1)

- **Contexte** : `AccountAutocomplete.svelte:265-269` ne capte *Entrée* que pour une liste ouverte avec un élément en
  surbrillance ; sinon le `<form>` de la facture ou de l'article part avec l'ancien compte pendant que le champ montre le
  texte tapé. L'orchestrateur demande de trancher pour un texte qui correspond à un compte unique, à plusieurs, ou à aucun.
- **Retenu** : dans le composant ; sans brouillon, inchangé (*Entrée* envoie, le champ montre la vérité) ; liste ouverte avec
  surbrillance, inchangé (choix visible, unique ou parmi plusieurs) ; liste fermée avec correspondances, la liste se rouvre ;
  aucune correspondance, le brouillon reste et un message le dit ; brouillon vide sans surbrillance, la résolution du `blur`
  (fonction commune `commitDraft()`). Une seconde *Entrée* envoie.
- **Écartées** : résoudre puis envoyer (la saisie jetée en silence au moment où la page change) ; choisir d'office la
  correspondance unique liste fermée (choix non montré, contraire à D2 de la 35-7) ; corriger dans chaque consommateur.
- **Réversible** : oui (une fonction).

### C-36-1-2 — Un message, clé neuve `common-account-no-match` (D2)

- **Contexte** : au cas « aucune correspondance », rien ne part ; sans signal, l'utilisateur ne sait pas pourquoi.
- **Retenu** : une clé globale `common-`, quatre locales, sous `common-account-invalid`, liée par `aria-describedby`,
  `aria-live="polite"`, effacée à la frappe, au choix, à l'effacement et au `blur`.
- **Écartée** : jeter le brouillon sans message (texte remplacé sans explication, faute de frappe à retaper).
- **Réversible** : oui.

### C-36-1-3 — Aucun consommateur modifié ; un test pour chacun des deux atteints (D3)

- **Contexte** : sept consommateurs ; seuls `InvoiceForm` et la fiche article sont dans un `<form>` (contrôle `<form>` sur
  chaque fichier et ses parents, cité dans la fiche).
- **Retenu** : correctif dans le composant ; un test Vitest par consommateur atteint (soumission implicite simulée par
  `requestSubmit` quand l'événement n'est pas empêché) et un E2E par écran ; les cinq stricts couverts par les tests du
  composant.
- **Écartée** : un test par consommateur strict (ils ne soumettent rien sur *Entrée*).
- **Réversible** : oui.

### C-36-1-4 — Le manuel ne change pas (D4)

- **Contexte** : le manuel ne décrit aucun geste clavier du champ de compte (`git grep` cité dans la fiche).
- **Retenu** : aucune modification ; le message se lit de lui-même ; comme la 35-7.
- **Écartée** : une phrase à `user-manual.tex:1159` (geste décrit pour un écran sur deux, fichier commun avec 36-3 et 36-5).
- **Réversible** : oui, à la demande de la revue.
