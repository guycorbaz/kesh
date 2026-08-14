# Story 22.4 : Un jeton PAT ne doit jamais atteindre une route d'administration — DÉCOUPÉE

## Status

split

# ⚠️ NE PAS IMPLÉMENTER DEPUIS CE FICHIER

**Cette story a été découpée le 2026-08-13. Elle n'a plus de corps : ni décisions, ni critères d'acceptation, ni tâches.** Tout ce qu'elle prescrivait vit désormais dans ses deux moitiés, et elles seules font foi :

| Story | Ce qu'elle porte |
|---|---|
| **[22-4a](22-4a-couche-anti-pat-routes-admin.md)** | Le **mécanisme** — la couche `route_layer` sur `admin_routes`, le code d'erreur `API_KEY_ADMIN_FORBIDDEN`, le test de complétude, la non-régression |
| **[22-4b](22-4b-frontiere-pat-dite-partout.md)** | La **frontière documentaire** — le guide d'intégration, le manuel, les énoncés de DC6, les traces de limitation, le CHANGELOG, la fermeture de #167 |

**Les deux se mergent dans UNE SEULE PR**, qui porte `closes #167` — le motif est au § *Découpage* de 22-4a : livrer le mécanisme sans la documentation publierait un logiciel dont le guide d'intégration **enseigne à exploiter** un trou fermé.

> ⚠️ **Ce fichier est vidé délibérément, et le précédent est dans ce dépôt.** La story `17-2-api-pat-integrations.md` est restée `ready-for-dev` après son propre découpage, gardant un corps complet — et elle est devenue un « vestige » que la story 22-4 a dû démêler quatre passes durant : sa décision **D4** a d'abord visé la mauvaise cible parce que deux documents portaient la même frontière avec des statuts différents. **Un fichier découpé qui garde son corps devient une seconde source de vérité, et elle dérive.** Ici, il n'en garde aucune : seul le journal des quatre passes, qui est de l'histoire et non de la prescription.

## Pourquoi le découpage

Quatre passes de `bmad-create-story validate`, avec rotation de modèles à chaque passe. Le trend :

| Passe | Modèles | CRIT | HIGH | MED | LOW |
|---|---|---|---|---|---|
| 1 | Sonnet ×3 | **1** | 3 | 6 | 3 |
| 2 | Haiku (aveugle) + Opus ×2 | 0 | 3 | 7 | 3 |
| 3 | Sonnet ×3 | 0 | **4** | 6 | 2 |
| 4 | Haiku (aveugle) + Opus ×2 | 0 | **4** | **9** | 8 |

Décomptes **dédupliqués** inter-lentilles. La sévérité maximale décroît une fois (`CRITICAL → HIGH`) puis **stagne sur trois passes**, et le volume de MEDIUM remonte en passe 4 : c'est le critère de **non-convergence réelle** de la § *Règle de splitting préventif* du `CLAUDE.md`.

**Le diagnostic est plus précis que le critère.** Ce n'est pas la story entière qui ne convergeait pas, c'est **sa frontière documentaire** : chaque passe, chaque lentille y trouvait un site de plus — d'abord le guide d'intégration entier, puis son tableau des codes d'erreur, puis trois lignes du manuel, puis quatre énoncés de DC6, puis sept énoncés de contrat dans d'autres specs. Le **mécanisme**, lui, a été confirmé exact par toutes les lentilles à chaque passe : références de ligne, décomptes, comportement d'axum, absence de route admin atteignable hors `admin_routes`.

⚠️ **La cause profonde est une forme de critère, et elle est réutilisable : AC6 ÉNUMÉRAIT.** Une clause qui énumère se lit comme close, et une énumération sur un corpus mouvant ne peut jamais l'être. La story **22-4b** renverse la méthode — un **critère décidable** (`D-a`), et l'inventaire n'est plus qu'un instantané que le critère supersède.

**Arbitrages de Guy** : le 2026-08-13 après la passe 3, ne pas découper mais **élaguer le Change Log** (trois des quatre HIGH portaient alors sur les comptes rendus, aucun sur le mécanisme) ; puis, après la passe 4, **découper** — la prémisse avait changé, trois des quatre HIGH touchant cette fois le mécanisme.

## Le journal des quatre passes

**Ce journal est de l'HISTOIRE.** Il dit ce que chaque passe a trouvé et où le correctif est parti. Il ne prescrit rien, et il n'est pas une source de vérité sur l'état du code — pour cela, 22-4a et 22-4b.

### Passe 1 — 2026-08-12 (Sonnet ×3)

**Le défaut central était une contradiction entre deux décisions de la story.** D2 crée un code d'erreur distinct, D3 conserve les trois `ensure_not_pat` : or `route_layer` enveloppe le service, donc la couche répond avant le handler et les trois tests existants reçoivent le **nouveau** code. La première rédaction affirmait l'inverse et avertissait qu'une modification de test signalerait « une couche posée trop haut » — **un avertissement qui aurait fait remettre en cause la bonne décision.**

**Le CRITICAL portait sur une circularité, et il avait raison** : AC1 exigeait d'énumérer les routes du routeur, avec pour repli une assertion contre « le nombre de routes du routeur ». `axum` n'expose aucune énumération ; ce nombre n'aurait pu venir que d'une seconde constante à la main — exactement le « quelqu'un doit se souvenir » que la story existe pour éliminer.

Trois autres corrections de fond : « route » n'était pas défini (chemins contre couples méthode-chemin) ; seul le scope `read-write` était prouvé ; et D4 amendait la mauvaise cible.

### Passe 2 — 2026-08-13 (Haiku sur la lentille aveugle, Opus sur les deux autres)

Collecte interrompue avant remédiation ; les correctifs ont été appliqués à la reprise, le même jour.

**Verdict sur la passe 1 : elle n'a pas menti, mais deux de ses corrections ont figé un périmètre incomplet en lui donnant l'apparence de l'exhaustivité.**

Les trois HIGH : le compteur mesurait les enregistrements quand le critère exige les couples ; la jambe `read-only` était un test muet sur vingt couples sur vingt-cinq, le gate de portée répondant avant la couche ; `docs/api-external.md` manquait des AC et des tâches. Les sept MEDIUM ont produit D6, l'amendement de D3 et de D4, et les exigences du mécanisme de comptage.

**Un enseignement de méthode** : le remède proposé peut être **moins bon que ce que le dépôt porte déjà**. Le relevé prescrivait un `grep -c` pour prouver l'i18n ; `kesh-i18n` a `client_number_labels_are_translated_in_all_four_locales`, qui asserte que la traduction diffère du français et ferme donc le repli silencieux. **C'est le grep de propagation post-patch qui l'a trouvé, pas la collecte.**

**Réfuté** : le LOW affirmant que le bras de `match` d'`errors.rs` était en `:1105` — il est en `:1104`.

### Passe 3 — 2026-08-13 (Sonnet ×3)

**Trois des quatre HIGH portaient sur les comptes rendus des passes précédentes**, un seul sur une clause de preuve du corps :

- **un précédent historique était inventé** — la passe 2 affirmait que `/admin/email-templates/{…}` serait « passée d'une à trois méthodes entre 20-1 et 20-2 » ; `git log -S` établit qu'elle est **née** avec ses trois méthodes. Et « six enregistrements multi-méthodes » en compte **cinq**. Les deux avaient été recopiés dans le corps **sous un commit déclarant que chaque fait du relevé avait été revérifié depuis la source** ;
- **la clause de preuve d'AC6 était satisfaisable sans faire le travail** : le `grep -rn "require_admin_role"` prescrit rend **sept** résultats avant tout amendement, aucun aux lignes visées ;
- **« quatre documents » pour trois documents et quatre énoncés** — correctif appliqué à T6 et non propagé à D4 ni AC6 ;
- **« les dix findings » pour une ventilation qui en compte treize.**

**Le MEDIUM le plus utile venait du source d'axum** : `route_layer` n'enveloppe que les routes **déjà présentes à l'appel**. Une route chaînée après les couches échappe **aux deux**, compile et ne panique pas.

### Passe 4 — 2026-08-13 (Haiku sur la lentille aveugle, Opus sur les deux autres)

**Cette fois, trois des quatre HIGH touchaient le mécanisme** — c'est ce qui a renversé l'arbitrage :

- **le compteur de complétude était aveugle à cinq familles de constructeurs.** Il énumérait sept constructeurs ; `axum` en exporte vingt-deux. Une route écrite `any(handler)` enregistre **neuf** méthodes, reste protégée, et **laisse le compteur à 25** : la protection tient, le **rappel** ne tient pas — or c'est le rappel qui est l'objet de la story ;
- **la quatrième assertion était bornée au bloc, alors que le trou de `route_layer` vit APRÈS lui** : près de six cents lignes de latitude jusqu'au `.merge(`, et `build_router` utilise déjà trois fois l'idiome de réaffectation ;
- **D4 réécrivait DC6 par remplacement, ce qui lui faisait perdre la gestion des clés** — et rendait fausses deux phrases de `17-2b` que la story déclarait « vérifiées comme restant vraies » ;
- **le tableau des codes d'erreur d'`api-external.md:267`** — celui dont le guide dit à l'intégrateur de se fier au `code` — n'était ni dans AC6 ni dans T6.

Neuf MEDIUM, dont : les cinq couples `HEAD` servis par les handlers `get` et comptés nulle part ; la chaîne discriminante de la passe 3 devenue **insatisfaisable avec le travail fait**, D4 la prescrivant avec balisage et la preuve la grepant sans ; la formule d'intersection présente à deux sites de plus ; `17-2c:129` rangé « hors sujet » alors qu'il énonce la faille au présent ; sept énoncés de contrat de `17-3*`/`14-2` jamais recensés ; l'absence de `closes #167`.

**Et deux findings portaient sur la remédiation de la passe 3 elle-même** : le commit d'élagage du Change Log a introduit « vingt-et-une vérifications », un nombre qui ne se recompte depuis aucune source — et c'était le chiffre qui justifiait de ne pas découper. Et le repli silencieux i18n était attribué à un test qui asserte en réalité l'autre branche.

⚠️ **La passe 4 a aussi réfuté une vérification de la passe 3** : « le 405 est engendré après les deux couches » est vrai des chemins exclusifs et **faux des chemins partagés** entre routeurs, où le fallback du dernier routeur mergé subsiste. Conservé en Dev Notes de 22-4a.

## Ce que les quatre passes ont confirmé sans défaut

À reprendre depuis 22-4a et 22-4b, pas depuis ici — mais le fait mérite d'être dit : **aucune route d'administration atteignable par un PAT hors `admin_routes` n'a été trouvée**, par aucune lentille, à aucune passe. `require_admin_role` n'est appliqué qu'à un seul site. Le changement de mot de passe self-service exige la vérification Argon2 ; les mutations d'onboarding sont fermées par leur garde d'étape ; `/api/v1/setup/admin` est fermé par son compteur d'utilisateurs ; le routeur `_test` est conditionné au mode test. **Le chemin d'attaque de #167 n'a pas de branche résiduelle.**

## Change Log

**2026-08-13 — découpée en 22-4a et 22-4b**, sur arbitrage de Guy, après quatre passes de `validate` et deux stagnations consécutives à HIGH. Le corps de ce fichier a été **retiré** — décisions, AC, tâches et Dev Notes vivent dans les deux moitiés — pour ne pas reproduire le vestige `17-2`. Seul le journal des passes subsiste, comme histoire.

**Aucun gate n'a été exécuté aux quatre passes** : la story n'a jamais eu de code.
