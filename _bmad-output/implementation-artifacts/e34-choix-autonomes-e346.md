# Epic 34 — choix autonomes du développement de la story 34-6 (clé e346, #953)

Choix tranchés en autonomie au développement (Opus 5.5, 2026-11-10), à côté de ceux de la fiche
(`e34-choix-autonomes-e343.md`, C-e343-6, 7 et 14). Mise à jour par union.

## C-e346-1 — un composant, deux emplois

- **Contexte.** La fiche veut « sous chaque champ en conflit » la valeur relue, un avis de bloc, et « un composant »
  `CompanyConflictNotice.svelte` (C-e343-7).
- **Retenu.** Le composant prend `notice`, `testid` et un `field` facultatif. Avec `field`, il rend la ligne posée sous ce
  champ (« Enregistré entre-temps : … » s'il est en conflit, « Repris de l'enregistrement entre-temps : … » s'il a été
  fusionné, rien sinon) ; sans `field`, l'avis du bloc. `data-testid` : `<testid>-changed-<champ>`,
  `<testid>-merged-<champ>`, `<testid>-notice`.
- **Écartées.** Deux composants (ligne et avis) : contraire à la lettre de la fiche, sans gain. Un avis unique listant les
  champs sous le bloc : contraire à « sous chaque champ ».
- **Réversible.** Oui.

## C-e346-2 — une sixième clé : l'avis de fusion

- **Contexte.** L'AC-7 nomme cinq clés « à fixer au développement ». Quand des champs sont repris sans qu'aucun ne soit en
  conflit (`kind = 'unchanged'`, `merged ≠ ∅`), la fiche veut un avis de fusion et non le « message actuel », et l'avis de
  remplacement ne vaut pas (rien ne sera remplacé).
- **Retenu.** `settings-company-conflict-merged-notice` : « Conflit de version : les champs que vous n'aviez pas modifiés ont
  repris la valeur enregistrée entre-temps. Vérifiez, puis enregistrez de nouveau. » L'avis de bloc porte la phrase de
  remplacement si `changed ≠ ∅`, celle de fusion si `merged ≠ ∅`, les deux si les deux.
- **Écartée.** Réemployer `settings-company-<bloc>-conflict` (« les données ont été rechargées, réessayez ») : il ne dit pas
  que des champs ont changé sous les yeux de l'Administrateur.
- **Réversible.** Oui.

## C-e346-3 — « déjà enregistrées » par la notification de succès

- **Contexte.** Cas `already-saved` : « l'édition se ferme, et une notification dit déjà enregistré ».
- **Retenu.** `notifySuccess(settings-company-conflict-already-saved)`, le même canal que l'enregistrement réussi des trois
  blocs ; `data` porte la société relue.
- **Réversible.** Oui.

## C-e346-4 — l'avis reste jusqu'au prochain conflit, à l'annulation, à la réouverture ou au succès

- **Contexte.** AC-6 bis nomme les remises à zéro : `start*Edit`, l'annulation, un enregistrement réussi, et le remplacement
  par un second conflit. Il ne dit rien d'un refus d'une autre nature (400) après un conflit.
- **Retenu.** L'avis n'est remis à zéro qu'aux points de l'AC-6 bis, et aussi quand une relecture échoue (AC-6 : « aucun
  avis de valeur »). Un refus 400 après un conflit laisse donc l'avis des valeurs relues à côté du message du refus : ces
  valeurs restent vraies, et l'Administrateur les a encore sous les yeux en corrigeant.
- **Écartée.** Effacer l'avis au début de chaque envoi : il disparaîtrait pendant l'envoi, et un refus 400 ferait perdre la
  valeur concurrente qu'enregistrer remplacera.
- **Réversible.** Oui.

## C-e346-5 — journaux et captures sous `kesh-gate-logs/e346/`

- **Contexte.** La fiche, écrite dans le groupe de validation `e343`, nomme `kesh-gate-logs/e343/rouges.log` et
  `kesh-gate-logs/e343/captures/`. Le prompt de développement dit `<clé>`, et la clé est `e346`.
- **Retenu.** `kesh-gate-logs/e346/` (rouges, mutations, captures, gates), pour ne pas mêler ce développement aux rapports
  de validation du groupe.
- **Réversible.** Oui (un déplacement de fichiers hors dépôt).

## C-e346-6 — l'E2E 4 des coordonnées est récrit

- **Contexte.** `company-contact-details.spec.ts`, test 4, attendait « Conflit de version » dans
  `settings-company-contact-error`. Avec la story, un téléphone modifié des deux côtés n'affiche plus le message d'erreur
  mais l'avis de conflit.
- **Retenu.** L'assertion passe à `settings-company-contact-conflict-notice` (dont le texte commence par « Conflit de
  version »), et le test vérifie en plus le téléphone de l'autre sous le champ. Un test 4 bis (AC-4, un seul champ changé)
  s'ajoute, comme le veut T5.
- **Réversible.** Oui.

## C-e346-7 — l'inventaire des écrans à version, relu à l'étape 0

- **Contexte.** La fiche demande à l'étape 0 de fermer l'inventaire par ses deux commandes et d'ouvrir une issue par écran
  confirmé.
- **Retenu.** `git grep -l "OPTIMISTIC_LOCK_CONFLICT\|status === 409" -- frontend/src ':!*.test.ts'` (32 fichiers) et
  `git grep -ln "version:" -- frontend/src ':!*.test.ts'` (51 fichiers) relancées au commit `665de151`. Le classement de la
  fiche tient. S'y ajoutent, lus au traitement de l'erreur : `settings/projects/+page.svelte` (édition :
  `mode.project.version` reste la périmée après un refus, le second clic rend 409 de nouveau ; archivage : relit la liste
  sans saisie à écraser) et `settings/vat-rates/+page.svelte` (`submitChange(mode.rate)` garde la ligne périmée : 409 de
  nouveau). Ni l'un ni l'autre ne porte le symptôme de #953. Les autres fichiers neufs de la seconde commande sont des types,
  des API ou des écrans sans formulaire à version (`McpPromptVersion.svelte` ne rend qu'une version de prompt ;
  `OpenItemsScreen.svelte` marque la liste périmée sur un 409). **Seul écran confirmé : `bank-accounts`**, issue demandée.
- **Réversible.** Sans objet (lecture).

## C-e346-8 — une version par édition ouverte, adoptée seulement si ses champs n'ont pas bougé

- **Contexte.** Revue de code P1, B-1 = E1-1 (MEDIUM). Les trois blocs envoyaient `data.company.version`, et la relecture
  d'un bloc remplaçait `data` pour toute la page. Un bloc resté ouvert enregistrait donc ensuite avec la version relue, et
  écrasait sans la montrer l'écriture concurrente portée sur ses propres champs : #953 par un autre chemin. Consigne de
  l'orchestrateur : un bloc ouvert n'envoie jamais une version plus récente que les valeurs qu'il a montrées, et pas de
  conflit parasite après l'enregistrement réussi d'un autre bloc de la page.
- **Retenu.** Les deux pistes proposées, combinées. Chaque bloc garde la version de sa copie (`emailVersion`,
  `contactVersion`, `addressVersion`), prise à `start*Edit` et mise à jour par sa propre relecture ; c'est elle qu'il
  envoie. Quand la page reçoit une société plus récente (succès d'un bloc, ou relecture après un 409), `syncOpenBlocks`
  fait adopter la nouvelle version à chaque édition ouverte **dont les champs n'y ont pas bougé** (`sameBlockValues`, aux
  normalisations de `companyEditConflict`). Les blocs ont des champs disjoints : l'enregistrement d'un autre bloc laisse
  les siens égaux, et le bloc adopte la version sans 409 parasite. Si ses champs ont bougé, il garde l'ancienne version, et
  son enregistrement reçoit son propre 409, jugé par `companyEditConflict` contre sa copie.
- **Écartées.**
  - La version par bloc seule : un 409 sans danger, mais à chaque enregistrement d'un autre bloc (« rechargées »).
  - Rejuger et afficher l'avis des autres blocs dès la relecture : un avis qui apparaît dans un bloc où l'Administrateur
    n'a pas cliqué, et un second chemin d'affichage à tenir à côté du premier.
- **Réversible.** Oui.

## C-e346-9 — les champs inactifs pendant l'envoi et la relecture

- **Contexte.** Revue de code P1, E1-3 (LOW) : les frappes faites pendant l'envoi et la relecture étaient jugées sans avoir
  été envoyées, puis écrasées par la saisie fusionnée.
- **Retenu.** Les champs des trois blocs sont `disabled` tant que `*Submitting` est vrai, comme leurs boutons l'étaient
  déjà. La saisie jugée est alors celle qui est partie.
- **Écartée.** Capturer la saisie au clic : les frappes ultérieures seraient encore écrasées par la saisie fusionnée.
- **Réversible.** Oui.
- *(Amendé le 2026-11-10, revue de code P2, B2 : `readonly` au lieu de `disabled`. Un champ désactivé pendant qu'il a le
  focus le perd, et ne le retrouve pas à la réactivation. En lecture seule, il le garde, et la frappe reste impossible.)*

## C-e346-10 — `data` et les versions des éditions ouvertes ne reculent jamais

- **Contexte.** Revue de code P2, P2-B1 = E2-1 = A2-5 (MEDIUM). Les réponses peuvent arriver dans le désordre. Une
  relecture partie avant le succès d'un autre bloc et revenue après portait une société plus ancienne. Elle remplaçait
  `data`, si bien que l'écran revenait à l'ancienne valeur du bloc qui venait de réussir. Elle faisait aussi adopter sa
  version, plus ancienne, à une édition ouverte inchangée, d'où un 409 parasite. Aucune écriture n'était perdue.
- **Retenu.**
  - Toute société reçue passe par `adoptCompany` : les succès des trois blocs, la relecture après un 409, le verrouillage
    et le déverrouillage de période (E2-3). Une société de version **inférieure** à celle de `data` est ignorée : elle ne
    touche ni `data` ni les versions. À version égale, elle remplace, parce que le verrou de période ne change pas la
    version.
  - `syncOpenBlocks` n'adopte qu'une version **strictement supérieure** à celle du bloc. Cette garde est une défense en
    profondeur : la version d'un bloc ne dépasse jamais celle de `data`, si bien que la mutation qui la retire (M14) est
    équivalente.
  - Le bloc en 409 garde sa logique : il juge contre la société relue et prend `r.base` et `r.version`, même quand
    `data` est plus récente. Sa copie et sa version restent cohérentes entre elles, et le pire est un 409 de plus, jamais
    un écrasement.
- **Écartée.** Verrouiller les trois blocs pendant une relecture : la course resterait entre deux enregistrements réussis,
  et l'Administrateur serait bloqué sans raison visible.
- **Réversible.** Oui.
