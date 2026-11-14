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
- **Retenu en P3, remplacé en P4** (V5-3) : désignation `bg-primary/10 ring-1 ring-inset ring-primary` (`--color-primary`, `app.css:12` ; `/10` déjà
  employé, `accounts/+page.svelte:504`) ; survol `hover:bg-border/60` (`--color-border`, `app.css:33`), gris sans filet.
  Contraste du texte sur la désignation mesuré au développement (`tests/e2e/helpers/contrast.ts`).
  *(Remplacé à la remédiation de la validation P4, V4-1 : ces deux fonds pâles se tenaient à 1,04:1 l'un de l'autre et à
  1,18:1 du fond de la liste ; seul le filet de 1 px distinguait la désignation. **Retenu désormais** : désignation
  `bg-primary text-primary-foreground` (`app.css:12`, `:14` ; ≈ 8,7:1 contre `--color-popover`, texte ≈ 8,7:1), survol
  `hover:bg-border` (`app.css:33`) sur les seules options non désignées (désignation contre survol ≈ 7,1:1 — « 7,3 » écrit en P4, corrigé au développement, V5-2) ; constantes
  `CLASSE_DESIGNEE`, `CLASSE_SURVOL` dans `account-autocomplete-classes.ts`, gardées par un test de jetons et de contrastes
  sur le patron de `button-destructive.test.ts` (AC8). Écartés : fond atténué + filet + barre gauche — trois classes à garder
  pour un indicateur que le fond plein donne seul.)*
- **Écartées** : `hover:bg-surface-alt` pour le survol (`#f8fafc` sur le fond blanc de la liste : à peine visible) ; définir
  `--color-accent`/`--color-muted` (portée de toute l'application) ; laisser la désignation invisible avec une issue (le
  principe (ii) de D1 reposerait sur un état que l'écran ne montre pas).
- **Réversible** : oui, deux classes.

### C-36-1-7 — La lecture du `@theme` extraite dans un module partagé (développement)

- **Contexte** : la garde d'AC8 doit lire le `@theme` d'`app.css` sans ses commentaires, exactement comme la garde de la
  28-1 (`button-destructive.test.ts`, fonctions locales `blocTheme`, `jeton`, `declare`, `motEntier`). Les recopier
  ferait deux copies d'une même lecture (règle DRY du `CLAUDE.md`).
- **Retenu** : `frontend/tests/e2e/helpers/theme.ts`, à côté de `contrast.ts` (même nature : fonctions pures servant des
  gardes Vitest) — `blocTheme`, `jetonDans`, `declareDans`, `motEntier`, et `jetonsDesClasses` (neuf : jetons de couleur
  d'une liste de classes). `button-destructive.test.ts` les importe sans changer ses assertions (ses `jeton`/`declare`
  locaux deviennent des enveloppes d'une ligne) ; la phrase d'en-tête de `contrast.ts` nomme la seconde garde.
- **Écartées** : recopier les fonctions (deux copies à garder d'accord) ; les placer sous `src/lib/shared/` (code de
  test dans l'arbre de production).
- **Réversible** : oui.

### C-36-1-8 — L'index dépend de `value` par un `$derived` primitif (développement)

- **Contexte** : premier passage vert d'AC1 hors le cas v : `rerender({ accounts: [...accounts] })` effaçait la
  désignation faite aux flèches. Cause lue : `@testing-library/svelte-core` (`props.svelte.js`) remplace tout l'objet de
  props à chaque `rerender`, si bien que la lecture de `value` est invalidée même quand sa valeur ne change pas. Un parent
  réel passe chaque prop par son propre accesseur, mais rien n'interdit à un parent de ré-évaluer ses props.
- **Retenu** : `const valeurLue = $derived(value)` ; l'index lit `valeurLue`, qui ne propage qu'un changement réel (égalité
  d'une valeur primitive). Le principe de D1 (« la chaîne des identifiants, non le tableau ») étendu à `value`.
- **Écartée** : réécrire le cas v sans `rerender` (le test cesserait de garder contre M23 tel que la fiche le décrit).
- **Réversible** : oui, une ligne.

### C-36-1-9 — Rouges E2E lisibles : attendre l'une des deux issues, puis asserter (développement)

- **Contexte** : AC5 a et b « attendent le message » ; sur le code d'avant correctif, cette attente expirait sans dire
  que le formulaire était parti.
- **Retenu** : attendre le message **ou** la conséquence du défaut (écran de détail de la facture ; plus aucun
  `role="dialog"` pour l'article) par `locator.or(…)`, puis asserter l'URL ou le dialogue, puis le message. Rouges
  constatés : URL du détail atteinte (a), dialogue refermé (b) ; a-bis : URL restée `/edit`. Aussi, a-bis ne vérifie pas
  l'absence de surbrillance dans le navigateur (portée par AC1 l) : la vérifier avant *Entrée* faisait rougir le test sur
  la surbrillance et non sur l'envoi.
- **Réversible** : oui.

### C-36-1-10 — Un brouillon qui reprend le libellé affiché désigne son compte (remédiation de la revue P1, E1)

- **Contexte** : revue P1, E1 (LOW) — le filtre ne comparait que le numéro (`startsWith`) et le nom (`includes`) ; un
  brouillon `3200 — Honoraires` (libellé retapé, ou corrigé d'une lettre) ou `3200 — Hon` ne correspondait à rien, et
  *Entrée* affichait « aucun compte ne correspond » sous un champ portant le libellé exact d'un compte. Prémisse vérifiée au
  code (`AccountAutocomplete.svelte`, filtre : `a.number.toLowerCase().startsWith(q) || a.name.toLowerCase().includes(q)`).
- **Retenu** : le filtre accepte aussi `labelOf(a).toLowerCase().startsWith(q)`, **seulement pour un brouillon**
  (`draft !== null`). Le compte est alors désigné par la saisie et *Entrée* le choisit (E1), comme pour un numéro tapé ; une
  nouvelle *Entrée* envoie. Sans brouillon, `query` est le libellé affiché : la condition laisse vide la liste d'un champ
  non touché, comme avant (sans elle, le focus d'un champ rempli ouvrirait une liste d'un seul compte, sur les sept
  consommateurs). Tests AC1 y (trois cas) ; mutations M27 (correspondance au libellé retirée) et M28 (condition
  `draft !== null` retirée).
- **Écartée** : traiter `draft === displayLabel` comme l'absence de brouillon (E2, envoi direct) — ne couvre ni un début de
  libellé (`3200 — Hon`) ni le libellé d'un autre compte, qui garderaient un faux message.
- **Réversible** : oui, une condition.

### C-36-1-11 — Aucun compte proposable : *Entrée* résout le brouillon comme le `blur` (remédiation de la revue P1, B2)

- **Contexte** : revue P1, B2 (LOW) — E4 posait le message si `accounts.length > 0` et, sinon, retenait le brouillon sans
  rien dire. Un plan comptable vide après chargement (ou une liste sans aucun compte du type attendu) laissait *Entrée*
  bloquée en silence ; une liste non vide sans compte proposable affichait « choisissez un compte dans la liste », faux.
  Le composant ne distingue pas « en cours de chargement » de « chargé, vide » (`accounts = []` dans les deux cas).
- **Retenu** : E4 (message) seulement si `active.length > 0` ; sinon **E4-bis** : `commitDraft()` (la règle du `blur` :
  strict → brouillon jeté, valeur réaffichée ; `allowClear` → texte libre jeté), liste fermée, index `-1`, aucun message.
  *Entrée* reste empêchée cette fois — le champ montrait un texte que la valeur contredit — et l'*Entrée* suivante envoie ce
  que le champ montre *(sauf pendant le chargement si une valeur est posée : le champ paraît vide alors que la valeur
  persistée part — exception préexistante d'E2, #1170 ; ajouté à la clôture, passe ciblée P2, R1)*. Pendant le chargement,
  le texte tapé est jeté comme il l'est déjà au `blur`. Tests AC1 m, m-bis (deux
  cas) ; mutations M12 (message même sans compte proposable), M33 (`commitDraft()` retiré d'E4-bis).
- **Écartées** : laisser l'événement suivre son cours (l'ancien compte partirait pendant que le champ montre la saisie :
  #1095 lui-même) ; un message propre au plan vide (clé i18n de plus, indiscernable du chargement sans prop nouvelle).
- **Réversible** : oui, une branche.
