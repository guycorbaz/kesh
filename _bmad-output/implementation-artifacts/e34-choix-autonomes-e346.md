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
