# Epic 34 — registre des choix autonomes, groupe e342 (34-3, 34-4, 34-9, 34-10)

*Ouvert le 2026-11-10 à la rédaction des fiches (Opus 5.5), worktree `/home/gcorbaz/devel/kesh-e34`, commit `a75835db`.
Mise à jour par union, jamais par écrasement.*

## 34-3 — « Écriture manuelle » (#967)

### C-e342-1 — Un avis quand la contrepartie est le compte débiteurs ou créanciers

- **Contexte** : retirer le filtre de classes fait proposer 1100 et 2000. Le manuel le disait impossible
  (`user-manual.tex:2961`). Une écriture manuelle sur 1100 laisse la facture due, sans lettrage : faux rattachement muet.
- **Retenu** : un avis non bloquant sous le sélecteur quand le compte choisi porte le rôle `Receivable` ou `Payable` (rôle
  explicite, Story 14-3a ; jamais déduit du numéro), qui renvoie au règlement depuis la fiche puis au rattachement.
- **Écartées** : exclure ces comptes (contraire à « tout compte imputable », décision de Guy) ; ne rien dire (le manuel seul,
  que personne ne lit au moment du geste) ; s'appuyer sur `letterable` (vrai pour tout actif ou passif hors banque : la caisse
  aussi, l'avis serait du bruit).
- **Réversible** : oui, un composant d'affichage.
- *(Précisé à la validation P1, F3-4 : **deux textes**, client et fournisseur, alignés sur `user-manual.tex:3947` et `:3948` —
  pour un fournisseur, la facture se saisit et se valide sans être payée depuis sa fiche, puis s'accepte à la réconciliation.)*

### C-e342-2 — Éclater écarte aussi le compte de la banque

- **Contexte** : `TransactionSplitModal` ne reçoit pas `bankLedgerAccountId` ; le filtre de classes l'écartait par accident.
- **Retenu** : la prop est passée, et une seule fonction pure filtre pour les deux dialogues (DRY).
- **Écartée** : laisser le serveur refuser en 400 après coup (le dialogue proposerait un compte refusé).
- **Réversible** : oui.

### C-e342-3 — Libellés : titre et bouton de validation suivent le bouton

- **Retenu** : bouton de ligne « Écriture manuelle » (de « Manuelle Buchung », en « Manual entry », it « Registrazione
  manuale ») ; titre du dialogue « Écriture manuelle » ; validation « Passer l'écriture » (« Buchung erfassen », « Post
  entry », « Registra »). Le libellé d'audit « Réconciliation manuelle effectuée » et le nom de l'action d'audit ne changent
  pas (piste existante).
- **Écartées** : garder « Réconciliation manuelle » et « Affecter » dans le dialogue (incohérent avec le bouton qui l'ouvre).
- **Réversible** : oui, catalogues. Les traductions de/en/it sont proposées par l'agent ; à relire par Guy s'il le souhaite.

### C-e342-4 — Les règles d'affectation restent aux charges et produits

- **Contexte** : `RuleFormModal.svelte:44-51` ne propose que `Expense` et `Revenue`.
- **Retenu** : hors périmètre (la décision de Guy vise les deux dialogues) ; issue ouverte : **#973**.
- **Réversible** : sans objet.

### C-e342-5 — Ordre 34-3 puis 34-4, et textes de la 34-4 au nouveau libellé

- **Retenu** : 34-3 passe la première ; les textes de la 34-4 nomment « Écriture manuelle » dès leur rédaction. Si les deux se
  développent en parallèle, l'intégration unit les catalogues.
- **Réversible** : oui.

## 34-4 — Transaction déjà comptabilisée (#828)

### C-e342-6 — Confirmation, et non ligne neutralisée (ex-Q2 de la v017-3c)

- **Contexte** : la v017-3c posait la question à Guy (« une ligne signalée ne propose-t-elle plus rien ? »).
- **Retenu** : #828, rédigée depuis, dit « une confirmation […] ; aucun refus serveur » : c'est la réponse. Les gestes restent,
  après confirmation.
- **Écartée** : neutraliser la ligne comme celle d'un lot en cours (un faux positif bloquerait un geste légitime).
- **Réversible** : oui.

### C-e342-7 — Texte du cas combiné

- **Retenu** : la phrase (a), puis « Si aucune de ces écritures ne lui correspond, elle est peut-être l'autre côté d'un
  virement entre vos comptes […] » : la consigne « rejetez-la » ne s'applique pas à une transaction qui a une écriture à
  laquelle se rattacher.
- **Écartée** : les deux phrases (a) et (b) juxtaposées, qui donneraient deux consignes contraires.
- **Réversible** : oui.

### C-e342-8 — Rejeter et Rattacher sans confirmation ; second côté couvert sans E2E à deux comptes

- **Retenu** : *Rejeter* et *Rattacher* sont les gestes que l'avertissement recommande : les confirmer découragerait le bon
  geste. Le cas (b) est prouvé par le test HTTP et Vitest ; un E2E à deux comptes bancaires coûte un montage lourd pour ce que
  le test HTTP voit déjà traverser la frontière (le champ est lu par l'écran dans le Vitest).
- **Réversible** : oui (un E2E s'ajoute sans rien changer).

### C-e342-9 — La mesure précède, et le remède est fait ici si la pente est mauvaise

- **Contexte** : la v017-3c renvoyait le remède à une dette ; la règle de l'epic demande de corriger les limites courtes
  découvertes dans sa zone.
- **Retenu** : mesure avant et après (1 000, 5 000, 20 000 écritures, 500 transactions, trois mesures, `--release`) ; si plus
  de 3 s à 20 000 ou pente plus que linéaire, le remède nommé (« une lecture par page ») est fait dans la story, sous condition
  d'égalité stricte avec les fonctions du dialogue ; à défaut, *Performance debt* et issue.
- **Écartée** : dette d'office (repousse un coût connu sur une page déjà à N requêtes par transaction).
- **Réversible** : oui.

## 34-9 — Rapport d'import du dossier (#969)

### C-e342-10 — Course `UNIQUE` : relecture par empreinte après le `rollback`

- **Retenu** : `find_by_company_hash` relu hors transaction, après le `rollback` ; s'il ne trouve rien, la ligne dit « déjà
  importé » sans lien.
- **Écartée** : `details` vide pour ce site (un des trois cas de doublon resterait sans lien, contre la décision de Guy).
- *(Étendu à la validation P1, F9-6 : la même relecture vaut au site `:577`, où l'état `discarded` lu avant la course est
  périmé.)*
- **Réversible** : oui.

### C-e342-11 — `details.movedTo` sur tout échec

- **Contexte** : le fichier est renommé `{stem}_{hash8}.{ext}` dans `failed/` ; l'utilisateur le cherche sous son nom
  d'origine. *(Annoté à la validation P1, F9-2 : le suffixe est l'empreinte ou un aléa de 8 caractères selon l'échec, et
  `movedTo` se fusionne dans un `details` existant — `FILE_TOO_LARGE` porte déjà `byteSize`, `maxBytes`.)*
- **Retenu** : le nom rangé, sans chemin absolu, sur toute entrée `failed[]` ; l'écran le cite pour tout refus et tout doublon.
- **Écartée** : ne le dire que pour un doublon (les autres refus partent au même endroit, sous un autre nom).
- **Réversible** : oui (champ additif).

### C-e342-12 — Ordre des lignes : par nom de fichier

- **Contexte** : `read_dir` rend un ordre arbitraire, et le rapport sépare acceptés et échecs.
- **Retenu** : tri par nom à l'écran, les deux listes fusionnées.
- **Écartée** : un champ d'ordre de traitement au serveur (contrat de plus pour un ordre sans sens pour l'utilisateur).
- **Réversible** : oui.

### C-e342-13 — Quand l'avertissement s'affiche

- **Contexte** : le commentaire de Guy dit « dès qu'au moins un fichier n'est pas "importé" » ; le fichier d'epic dit « dès
  qu'un fichier échoue ». Un document « importé sans les données du QR » est entré dans la liste.
- **Retenu** : avertissement quand `failed` (doublon ou refus) ou `warnings` (fichier non traité) n'est pas vide ; succès sinon.
  Un document importé sans QR ne déclenche pas l'avertissement : sa ligne le dit.
- **Écartée** : avertir aussi pour un import sans QR (fréquent pour des scans : l'avertissement deviendrait habituel et ne se
  lirait plus).
- **Réversible** : oui, une condition. **Signalé à Guy** dans le rapport.
- *(Précisé à la validation P1, F9-8 et #974 : `warnings` est remplacé, pour l'écran, par `skipped[]` et `truncatedAt` ;
  `n = failed.length + skipped.length` ; une seule notification si le rechargement de la liste échoue aussi.)*

### C-e342-14 — Où mène le lien d'un doublon

- **Retenu** : `to_complete` → ancre `#imported-<id>` de la ligne dans la liste de la même page ; `completed` avec facture →
  `/supplier-invoices/<id>` ; `completed` sans facture ou `discarded` → texte sans lien (aucun écran ne liste les documents
  retirés).
- **Écartée** : un écran de détail du document importé (n'existe pas ; hors périmètre).
- **Réversible** : oui.

## 34-10 — Graphiques en couleurs unies (#970)

### C-e342-15 — Jetons et palette

- **Retenu** : `--color-chart-revenue` = `--color-success` `#16a34a`, `--color-chart-expense` = `--color-destructive`
  `#b91c1c` ; postes rouge `#b91c1c`, vert `#16a34a`, bleu `#1e40af`, violet `#7c3aed`, cyan `#0891b2` ; « Autres »
  `--color-text-muted`. Tous ≥ 3:1 sur blanc ; vert et rouge écartés en clarté de 1,96:1.
- **Écartées** : `--color-error` `#dc2626` pour les dépenses (1,47:1 seulement contre le vert : plus confondu en vision
  daltonienne) ; ambre ou orange en quatrième (plus proche du rouge en simulation de deutéranopie).
- **Réversible** : oui, valeurs de jetons.
- *(Précisé à la validation P1, F10-2 : jetons écrits en hexadécimal, non en alias ; le module TypeScript garde des
  constantes hexadécimales, nécessaires aux tests de contraste et au rendu sous jsdom ; un test les tient égales.)*

### C-e342-16 — Liseré blanc entre les parts

- **Retenu** : un trait de la couleur de fond entre les secteurs : chaque limite se voit, quelle que soit la paire de teintes.
- **Réversible** : oui.

### C-e342-17 — Le centre porte `totalExpenses`

- **Contexte** : `totalExpenses` (compte de résultat, somme des barres) et `expensePostsBase` (somme des soldes positifs,
  base des parts) diffèrent quand un compte de charges est au solde créditeur.
- **Retenu** : `totalExpenses`, qui est « le total des dépenses » que Guy demande et qui recoupe les barres ; la note
  existante dit alors sur quelle base portent les parts. *(Annoté à la validation P4 ciblée, F5 : prémisse fausse à sa date —
  la note existante, `homepage-charts-credit-balance-note-one` / `-other`, ne dit que le nombre de comptes écartés. La base des
  parts se dit par une phrase jointe après elle, clé neuve `homepage-charts-pie-base-note`, et dans le `<desc>` par
  `homepage-charts-pie-desc-base` (C-e342-25).)*
- **Écartée** : `expensePostsBase` (égale la somme des parts, mais ne recoupe ni les barres ni le compte de résultat).
- **Réversible** : oui. **Signalé à Guy** dans le rapport.

### C-e342-18 — Dire ce que le rouge/vert ne garantit plus

- **Retenu** : la fiche et le Change Log disent que la paire reste confondue par la teinte en vision deutéranope ou protanope,
  et ce qui la sépare (clarté, position, légende, tableau, survol) ; capture en simulation à l'appui.
- **Réversible** : sans objet.

## Ajouts de la validation P1 (2026-11-10)

### C-e342-19 — #974 intégrée à la 34-9

- **Contexte** : Guy veut « chaque fichier » au rapport ; un fichier instable ou disparu sortait en chaîne française
  (`warnings[]`), sans nom structuré (#974, ouverte à la demande de la fiche). La correction est de la taille du `details`
  déjà ajouté (finding F9-4).
- **Retenu** : `skipped: [{ fileName, reason: UNSTABLE | VANISHED }]` et `truncatedAt: number | null`, additifs, traduits par
  l'écran dans les quatre locales ; `warnings[]` gardé tel quel pour les clients d'API, déprécié. La PR porte `closes #974`.
- **Écartées** : garder #974 à part (le rapport « chaque fichier » resterait incomplet dans la même release) ; retirer
  `warnings[]` (changement de contrat d'une route documentée aux clés d'API).
- **Réversible** : oui (champs additifs).

### C-e342-20 — Le PDF du manuel se régénère à l'intégration

- **Contexte** : 34-3, 34-4, 34-9 et 34-10 touchent `user-manual.tex` (34-9 aussi `admin-manual.tex`) ; un PDF ne se fusionne
  pas (finding X-1).
- **Retenu** : chaque story change le `.tex` ; le PDF se régénère **une fois à l'intégration**, et s'y contrôle aplati. Une
  story peut le régénérer pour se relire, sans le commiter. Le `CLAUDE.md` (PDF à jour au push et à la release) est tenu par
  la PR d'intégration.
- **Écartée** : un PDF régénéré par story (conflits binaires à chaque fusion, PDF d'une story écrasé par la suivante).
- **Réversible** : oui.

### C-e342-21 — Pas de sélecteur Fluent dans les textes lus par l'écran

- **Contexte** : le client reçoit les messages pré-résolus sans arguments (`crates/kesh-i18n/src/loader.rs:130-165`) et
  `i18nMsg` ne remplace que `{ $nom }` (`frontend/src/lib/shared/utils/i18n.svelte.ts:19-27`) : un sélecteur arriverait figé
  (findings F4-1, F9-1).
- **Retenu** : 34-4, deux clés `-one`/`-other` choisies par le Svelte ; 34-9, un résumé composé par segments, chacun au pluriel
  par « (s) » ; un test lit le catalogue réel.
- **Réversible** : oui.

## Ajouts de la validation P2 (2026-11-10)

### C-e342-22 — Sections du CHANGELOG

- **Contexte** : le prompt commun nomme *Ajouté* et *Corrigé* ; l'orchestrateur admet *Changé* pour un comportement modifié
  (finding P2-X-1).
- **Retenu** : 34-3 *Changé* (un geste existant change de nom et de portée) ; 34-10 *Changé* (le rendu d'une fonction existante
  change) ; 34-4 *Ajouté* (un signal neuf) ; 34-9 *Corrigé* (#969, #974 : un défaut de signalement).
- **Écartée** : tout en *Ajouté* (« Écriture manuelle » et les couleurs unies ne sont pas des fonctions neuves).
- **Réversible** : oui, à l'intégration.

### C-e342-23 — Ce que la 34-11b réécrit de la 34-4

- **Contexte** : après la 34-11b, `matchedElsewhereCount ⊂ entryCount` ; le cas (b) de la 34-4 ne s'afficherait plus et (c)
  dirait faux (finding P2-4-1).
- **Retenu** : la condition d'affichage (a) / (b′) / (c′) et les textes (b′), (c′) sont écrits à l'AC-5 de la 34-4, à transmettre
  par l'orchestrateur à l'agent de la 34-11b ; la fiche 34-11b n'est pas modifiée par ce groupe.
- **Réversible** : oui.

### C-e342-24 — Taille du total au centre de l'anneau : règle pure

- **Contexte** : jsdom n'a pas de mise en page (finding P2-10-3).
- **Retenu** : une fonction pure de la longueur du texte, testée en Vitest ; aucune mesure de rendu.
- **Écartée** : une mesure en E2E (`boundingBox`), coûteuse pour un montant que le harnais n'atteint pas.
- **Réversible** : oui.

## Ajouts de la validation P3 (2026-11-10)

### C-e342-25 — Le `<desc>` de l'anneau : deux clés existantes au total, une clé jointe pour la base

*(Annoté à la validation P6, F3 : l'état retenu compte **deux** clés jointes pour la base — `homepage-charts-pie-desc-base` dans
le `<desc>`, `homepage-charts-pie-base-note` visible — sous la condition `hasPosts` et `totalExpenses ≠ expensePostsBase`.)*

- **Contexte** : le `<desc>` existe en deux clés selon le nombre de comptes (`homepage-charts-pie-desc-one`,
  `homepage-charts-pie-desc`), toutes deux « pour un total de { $base } » ; la remédiation P2 en prescrivait deux autres sans
  dire ce qu'il advenait de celles-là (finding P3-10-1).
- **Retenu** : les deux clés existantes disent le total des dépenses (`totalExpenses`) ; une clé neuve
  `homepage-charts-pie-desc-base` (« Parts calculées sur { $base }. ») est jointe par le Svelte quand la base diffère.
  *(Complété à la validation P5, P5-5 : sous la même condition, la phrase visible `homepage-charts-pie-base-note` est jointe
  après la note des comptes créditeurs — C-e342-17 annotée.)*
- **Écartée** : quatre clés (nombre de comptes × égalité), qui doublent les textes à traduire pour une seule phrase de plus.
- **Réversible** : oui.
