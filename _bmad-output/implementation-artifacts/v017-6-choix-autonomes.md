# Release 0.17.0 — registre des choix autonomes de la Story v017-6

Registre propre à la story (`docs/agents-orchestration.md` § 1), à verser **par union** au registre de la release au
merge. Fiche : `v017-6-petites-corrections.md`. Numérotation `C-v017-6-k`. Issues : #832 (P4), #824 (P3), en `refs`
seulement. Numéros de ligne de `8e126cfd`. Chaque choix : contexte, option retenue, alternatives écartées,
réversibilité.

---

### C-v017-6-1 — #832 : corriger aussi l'IBAN et le QR-IBAN, quatre sites de plus

- **Contexte** : `CoreError::InvalidIban` et `InvalidQrIban` portent leur propre « … invalide : » (`errors.rs:17-23`), et
  quatre routes (`onboarding.rs:601, 607` ; `bank_accounts.rs:320, 327`) le préfixent du même mot. L'issue ne cite que l'IDE.
- **Retenu** : les cinq sites, même remède (`e.to_string()`). Le mandat demande « tous les sites » ; le défaut est
  identique et mécanique.
- **Écartées** : (a) l'IDE seul, les IBAN laissés doublés — contre la lettre du mandat, et le même doublon réapparaîtrait
  sur le compte bancaire ; (b) une issue distincte pour l'IBAN — plus coûteux que le correctif.
- **Réversible** : oui.

### C-v017-6-2 — #832 : ne pas préfixer, plutôt que préfixer par le nom du champ

- **Contexte** : l'issue propose « ne pas préfixer, ou préfixer par le nom du champ seulement ».
- **Retenu** : ne pas préfixer. Le message de `kesh-core` nomme déjà le champ (« Numéro IDE », « IBAN », « QR-IBAN »).
  Un préfixe de champ (« IDE : ») le doublerait encore.
- **Écartées** : « Coordonnées : … » ou « Champ IDE : … » — plus de texte en dur, à traduire un jour (#710), sans gain.
- **Réversible** : oui.

### C-v017-6-3 — #824 : corriger aussi `user-manual.tex:3732` et `README.md:53`

- **Contexte** : l'inventaire de la phrase fausse rend quatre sites, l'issue en nomme un. Deux autres répètent « la
  langue est fixée / choisie à l'onboarding ».
- **Retenu** : les corriger dans la même story (règle *Propagation post-patch* : greper le symptôme sur tout le dépôt).
  Le README est un support à auditer à chaque commit (CLAUDE.md, *Synchroniser le planning du README*).
- **Écartées** : n'agir que sur le § de l'issue — laisserait le manuel se contredire (§ et limites).
- **Réversible** : oui.

### C-v017-6-4 — #824 : la note « sélecteur à venir » reste, son origine change

- **Contexte** : la `keshnote` annonce un sélecteur de langue pour plus tard. Ce qui est faux est « fixée à l'onboarding ».
- **Retenu** : garder l'annonce du sélecteur, retirer l'origine fausse ; dire que la langue ne se change pas depuis
  l'application.
- **Écartées** : supprimer la note — perd une information vraie (limite connue, reprise à la ligne 3732).
- **Réversible** : oui.

### C-v017-6-5 — Le manuel d'administration n'est pas modifié

- **Contexte** : `admin-manual.tex:757` dit « Locale de l'instance (fr/de/it/en) » : exact, mais ne dit pas que c'est la
  langue de l'interface.
- **Retenu** : ne pas y toucher. Le mandat dit qu'il « le dit correctement » et le cite ; la formulation du manuel
  utilisateur s'y rattache. Préciser « de l'interface » à la ligne 757 est une amélioration, non un défaut — à demander
  si le validateur la juge nécessaire.
- **Écartées** : modifier le registre des variables — un PDF d'administration de plus à régénérer pour un gain faible.
- **Réversible** : oui.

### C-v017-6-6 — Assertions d'égalité exacte, entrées synthétiques minimales

- **Contexte** : les tests existants (`starts_with`, code seul) restent verts avant et après : aucun ne détecte la
  régression.
- **Retenu** : égalité exacte du message pour l'IDE et l'IBAN/QR-IBAN de l'assistant ; pour le compte bancaire, comptage
  de « IBAN invalide » (une seule fois), le `starts_with` existant conservé. Valeurs relevées au serveur, non devinées.
- **Écartées** : un test unitaire de la fonction — la répétition naît à l'appel de la route, pas dans `kesh-core`.
- **Réversible** : oui.

### C-v017-6-7 — Pas de Vitest, d'E2E ni de capture

- **Contexte** : aucun fichier de `frontend/` ne change ; le texte du toast est plus court, aucun contrôle ne bouge.
- **Retenu** : gate backend complet seulement (`kesh-api` touché), contrôle du PDF aplati pour le manuel.
- **Écartées** : une capture du toast — la règle vise un contrôle visible ajouté ou modifié ; un texte raccourci n'en est pas un.
- **Réversible** : oui.

### C-v017-6-8 — Le décompte de l'intro du CHANGELOG n'est pas touché

- **Contexte** : l'intro de la 0.17.0 dit « six stories » (`CHANGELOG.md:13`). Cette story en ajoute une sans en ajouter
  la mention.
- **Retenu** : n'ajouter que les deux puces *Corrigé* ; l'intro se recompte à la PR de release (qui seule voit l'ensemble).
- **Écartées** : la réécrire ici — conflit de fusion certain avec les autres stories de la release.
- **Réversible** : oui.
