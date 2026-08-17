# Story 22.2 : Prévenir les doublons à la saisie d'un contact

## Status

ready-for-dev

## Story

**As a** utilisateur qui saisit un contact,
**I want** que Kesh me signale, pendant que je tape, qu'un contact proche existe déjà,
**so that** je reprenne la fiche existante au lieu d'en créer une seconde — au moment où cela ne coûte encore rien.

Ferme **#301**. Cinquième story livrée de l'**Epic 22 « Technical Debt Closure »** (après 22-4a, 22-4b, 22-5 et 22-1).

## Contexte

Le même client finit par exister deux fois : saisi une première fois, ressaisi plus tard sans qu'on s'en souvienne. Rien ne le signale au moment où c'est encore gratuit — **avant** l'enregistrement.

**Le moment est le bon, et il ne se représentera pas.** Kesh est déployé mais **ne tient pas encore les comptes réels** : le jalon « Première clôture d'exercice tenue dans Kesh » est ouvert. Il n'existe donc **aucun parc de doublons à réparer**.

C'est ce qui rend l'arbitrage net : la prévention coûte le même prix aujourd'hui qu'elle coûtera dans deux ans, mais elle **évite** la dette au lieu de la rembourser. La fonction de fusion (Story 22-3, #300) est une réparation — elle reste en veille, et le mieux serait qu'elle n'ait jamais à servir.

**Ce que la Story 22-1 a changé depuis la rédaction de cette note (12 août).** Elle a livré `kesh_core::text::canonical_key` et la colonne `contacts.client_number_canonical` : *deux numéros de client identiques à l'œil sont désormais le même numéro*. La présente story est le **pendant à la saisie** de ce qui y a été fait à l'enregistrement — sauf que le nom, lui, n'a et n'aura **aucune** contrainte d'unicité (D1). Ce qui vaut pour un identifiant ne vaut pas pour un nom.

## Décisions

**D1 — Signaler, jamais bloquer.** Deux clients peuvent légitimement porter des noms très proches : deux sociétés d'un même groupe, deux homonymes, un père et son fils. Le dispositif **informe** et laisse l'utilisateur décider. Un blocage produirait des contournements — un espace ajouté au nom — qui salissent le carnet plus sûrement qu'un doublon assumé.

**D2 — Le numéro IDE est un signal fort, le nom un signal faible.** Ils n'appellent pas le même avertissement :

| Ce qui correspond | Avertissement |
|---|---|
| **Numéro IDE identique** | Franc : c'est un identifiant d'État, deux entités ne le partagent pas. Presque certainement le même contact. |
| **Nom proche** | Nuancé : *« un contact au nom proche existe »*, avec de quoi le reconnaître — nom complet, localité, numéro de client. |

**D3 — Réutiliser la recherche existante, ne pas en inventer une seconde.** Le carnet se cherche déjà par index FULLTEXT sur `name`, complété d'un `LIKE` sur l'email et le numéro de client (Story 16-3b). C'est ce chemin qu'il faut réemployer. ⚠️ **Il porte DEUX branches** dans `push_where_clauses` — celle du terme échappé vide et celle du cas courant : **les deux, ou aucune**.

**D4 — Hors périmètre, explicitement.** Trois cas voisins relèvent d'un autre traitement, et les confondre ferait dériver cette story :

- l'**appariement automatique** à l'import de documents — gouverné par la règle « un appariement propose, il ne crée jamais » du `CLAUDE.md` ;
- le **rachat d'entreprise** (#302) — ce n'est pas un doublon mais une **succession datée** entre deux entités distinctes, avec deux numéros IDE ;
- la **recherche mid-word** — « barde » ne remonte pas « Dubarde ». C'est une régression assumée depuis la Story 7-4 / KF-005 (BOOLEAN MODE, wildcard de préfixe seulement), gardée par `test_search_no_longer_matches_mid_word`. Cette story n'y touche pas.

**D5 — Aucune route nouvelle ; une seule extension du chemin existant.**

`GET /api/v1/contacts?search=…&limit=…` fait déjà **tout** ce qu'AC1 demande, et il faut résister à l'envie d'en écrire une autre : il est scopé société (`current_user.company_id`, `contacts.rs:591`), il exclut les archivés par défaut (`push_where_clauses:158`), et il cherche `name` en FULLTEXT. **Rien à ajouter pour AC1.**

Pour AC2, il manque une chose, et une seule : **`ide_number` n'est pas cherché**. `push_where_clauses` couvre `name` (FULLTEXT), `email` et `client_number` (LIKE) — pas l'IDE. Deux voies :

- **(a) ajouter `ide_number` aux DEUX branches `LIKE` de `push_where_clauses`** — retenue. Un seul chemin de recherche, une seule garde de scoping, deux tests.
- **(b) une route dédiée** `GET /contacts/check-duplicate` — écartée. Elle duplique un chemin de recherche (contre D3) et rouvre le risque que T1 nomme : une route neuve est précisément celle qui oublie le `company_id`.

⚠️ **(a) élargit un comportement visible** : chercher un IDE dans le carnet y remontera désormais son contact. C'est cohérent avec ce que la 16-3b a fait du numéro de client, et le manuel utilisateur vante déjà « remonter d'une facture papier au contact ». **Mais c'est un changement d'UX, donc il est porté en Q1 pour arbitrage** — pas décidé ici.

**D6 — La garde anti-écrasement est un compteur de génération, PAS un `AbortSignal`.**

⚠️ **Piège vérifié dans le code, et cette story est le premier appelant qu'il concerne.** `api-client.ts:259-268` :

```ts
async function fetchWithTimeout(url: string, init: RequestInit): Promise<Response> {
	const controller = new AbortController();
	const timeout = setTimeout(() => controller.abort(), REQUEST_TIMEOUT_MS);
	…
	return await fetch(url, { ...init, credentials: 'include', signal: controller.signal });
```

Le spread place `signal` **après** `...init` : tout `signal` passé par l'appelant est **écrasé en silence**. Un code qui crée un `AbortController`, le passe à `apiClient.get` et croit annuler ses requêtes **n'annule rien** — et rien ne le lui dit, ni compilation, ni test, ni erreur au runtime. Le commentaire de `api-client.ts:250-258` documente exactement ce cas et désigne d'avance le futur appelant qui aurait besoin d'une annulation externe. **C'est nous.**

Le pattern éprouvé du dépôt est le **compteur de génération** de `ContactPicker.svelte:37,57,61,65,69` :

```ts
let searchSeq = 0;
async function runSearch(q: string) {
	const seq = ++searchSeq;
	const r = await listContacts({ search: q, … });
	if (seq !== searchSeq) return;   // ← une réponse tardive ne dit plus rien
	results = r.items;
}
```

Il **ne coupe pas** la requête réseau — il rend sa réponse inoffensive, ce qui est très exactement ce qu'AC4 demande. Alternative écartée : composer les signaux par `AbortSignal.any([init.signal, controller.signal])` dans `api-client.ts`. Ce serait modifier le chemin HTTP de **toute** l'application pour une seule page — et le gain (quelques requêtes économisées sur un carnet local) ne paie pas ce rayon d'impact.

**D7 — Le seuil de déclenchement est 3 caractères, et ce n'est pas une politique d'ergonomie.**

Mesuré sur la base de dev le 2026-08-17 :

```
innodb_ft_min_token_size	3
```

En dessous de trois caractères, `MATCH(name) AGAINST(…)` ne rend **rien**. Un dispositif qui se déclencherait à la deuxième frappe serait **muet sans que rien ne le signale** — et un développeur qui teste à la main sur « Du » conclurait que sa recherche est cassée. Le seuil est donc d'abord une **contrainte du moteur**, et accessoirement une bonne politique (moins de bruit).

Note : `innodb_ft_enable_stopword = ON`, mais la liste InnoDB par défaut est anglaise et ses entrées font pour l'essentiel moins de trois caractères — sans effet pratique ici. À ne pas confondre avec `ft_min_word_len`, qui est la variable **MyISAM** et ne s'applique à aucune table de Kesh.

**D8 — Le champ déclencheur dépend du type de contact, et l'ignorer rend le dispositif à moitié mort.**

Le formulaire n'a pas un champ « nom » mais deux formes :

| Type | Champs | `name` |
|---|---|---|
| `Entreprise` | `#form-name` (raison sociale) | saisi tel quel |
| `Personne` | `#form-firstname` **+** `#form-lastname` | **recomposé côté serveur** : `format!("{f} {l}")`, `contacts.rs:362-380` |

Un dispositif branché sur le seul `#form-name` serait **entièrement inopérant pour les personnes physiques** — sans rien casser, sans faire échouer la compilation, et en restant vert sur tout test qui ne crée que des entreprises. Le terme de recherche est donc :

```ts
const term = formContactType === 'Personne'
	? `${formFirstName} ${formLastName}`.trim()
	: formName.trim();
```

**D9 — Où vit le code, et le piège du lint i18n.**

⚠️ `lint-i18n-ownership.js:154-163` compare le **dossier de feature** au **premier segment de la clé**. Le dossier est `contacts` (pluriel), les clés sont `contact-*` (singulier) : `getNamespace('contact-x') === 'contact' !== 'contacts'` ⇒ **toute** clé `contact-*` employée depuis `src/lib/features/contacts/` est une violation. C'est pour cela que `ContactCard.svelte` et `ContactPersonsManager.svelte` sont inscrits en dur dans `KNOWN_VIOLATIONS` (lignes 99-111) — c'est la dette #30, pas un accident.

Retenu : **la logique pure dans un `.ts` sans aucune clé i18n**, le balisage et les clés restant dans `routes/(app)/contacts/+page.svelte`, qui est **hors du champ du lint** (`FEATURES_PATH` ne couvre que `src/lib/features`). Bénéfice double : le lint n'a rien à connaître, et la logique (seuil, tri, exclusion de soi, vérification de l'IDE) devient testable au vitest **sans DOM**.

Écarté : poser un composant `.svelte` dans `features/contacts/` et allonger `KNOWN_VIOLATIONS` — cohérent avec l'existant, mais nourrit une dette que l'Epic 22 est censé refermer.

**D10 — L'IDE se vérifie sur le champ, pas sur le fait qu'un résultat remonte.**

La recherche par `search=` est un `LIKE %…%` sur **plusieurs colonnes**. Un contact peut donc remonter parce que la chaîne figure dans son nom ou son email, **sans porter cet IDE**. Émettre le franc avertissement de D2 sur le seul fait qu'un résultat existe produirait un message **faux et péremptoire** — le pire des deux mondes, puisque c'est le signal auquel on demande à l'utilisateur de faire confiance.

Le franc avertissement n'est émis que si un résultat vérifie **deux** conditions :

```ts
c.ideNumber === normalizeIdeForApi(formIde) && c.id !== editing?.id
```

⚠️ **La seconde n'est pas un raffinement, c'est la condition sans laquelle la sonde crie à faux à CHAQUE édition.** `openEdit` pré-remplit `formIde` avec l'IDE du contact lui-même (`+page.svelte:256`, `formIde = formatIdeNumber(c.ideNumber)`) : à l'ouverture d'une fiche pourvue d'un IDE, le champ est **déjà** `validateIdeFormat`-valide, et la sonde retrouverait **le contact qu'on est en train de modifier**. L'avertissement franc — celui auquel on demande à l'utilisateur de faire confiance (D2) — annoncerait donc « ce numéro IDE existe déjà » en désignant sa propre fiche, sur le chemin le plus banal du formulaire.

C'est le même défaut qu'AC7 écarte pour la sonde nom, et il fallait le dire **deux fois** : les deux sondes ne partagent pas leur code, et le raisonnement ne se propage pas tout seul d'une à l'autre.

Et l'IDE ne se cherche qu'une fois **complet** : `validateIdeFormat` (`contact-helpers.ts:28`, regex `^CHE[0-9]{9}$` après retrait des séparateurs) est la porte d'entrée. Interroger sur `CHE-12` ne sert à rien.

**D13 — Deux sondes, deux minuteries, deux compteurs.** Elles ne partagent **rien** de leur état : chacune porte sa propre paire `(timer, compteur de génération)`. Une paire unique factorisée entre les deux ferait qu'une réponse tardive de l'une **invalide la fraîcheur de l'autre** — l'un des deux avertissements ne s'afficherait jamais, sans erreur, sans test rouge, et sans que rien ne le signale. `ContactPicker.svelte` ne peut pas servir de modèle sur ce point : il n'a qu'**un** flux.

**D14 — L'état des sondes se remet à zéro à l'OUVERTURE du formulaire, pas à sa fermeture.** Le dialogue se ferme depuis **trois** sites (`+page.svelte:352`, `:358`, `:846`) et n'est **jamais démonté** — le nettoyage de `onMount:113` ne joue qu'à la sortie de la page. Poser le nettoyage sur la fermeture obligerait donc à le poser trois fois, et un quatrième site futur le contournerait en silence. `openCreate()` et `openEdit()` (`+page.svelte:216-264`) réinitialisent déjà champ par champ : les minuteries, les compteurs et les propositions affichées s'y ajoutent, au même titre que le reste. Sans quoi une réponse armée avant un « Annuler » vient s'afficher sur la fiche **suivante**.

**D11 — La localité s'affiche, elle ne se cherche pas.** *(Réponse à la question ouverte 1 de la note du 12 août.)* Une correspondance « nom **et** localité » serait un signal plus fort — mais l'obtenir demanderait une dimension de requête supplémentaire dans `push_where_clauses`, alors que le même service est rendu **à coût nul** en affichant la localité dans la proposition : c'est l'utilisateur qui discrimine, et il le fait mieux qu'un seuil. La requête porte donc sur le nom seul ; la localité, le numéro de client et le nom complet sont là **pour reconnaître**.

**D12 — Les deux sondes n'ont PAS la même politique d'archivage, et ce n'est pas une incohérence.**

⚠️ **Les deux contraintes d'unicité de `contacts` sont asymétriques, délibérément**, et la migration `20260810000001_contacts_client_number.sql:39-49` explique pourquoi en toutes lettres :

| Contrainte | Forme | Un contact archivé… |
|---|---|---|
| `uq_contacts_company_ide` | **plate** (`20260414000001:23`) | **garde son IDE à vie** — un IDE est attribué par l'État, jamais réattribué |
| `uq_contacts_company_client_number` | **partielle** (colonne générée `client_number_uniq`) | **libère son numéro** — étiquette interne, recyclable |

La conséquence pour AC2 est directe et sévère. Un contact **archivé** occupe toujours son IDE : ressaisir cet IDE sur une fiche neuve donne un `409 IDE_ALREADY_EXISTS`. Or une sonde qui interroge avec le défaut `includeArchived: false` **ne verrait rien** et se tairait — l'utilisateur remplirait toute la fiche pour se heurter à un refus dont **le coupable est invisible dans son carnet**. Et il n'aurait aucun recours : un contact archivé n'est pas modifiable (`IllegalStateTransition`) et **aucune route de désarchivage n'existe**.

D'où :

- **la sonde IDE (AC2) interroge `includeArchived: true`**, et l'avertissement **dit que le porteur est archivé** quand il l'est — c'est précisément le cas où l'utilisateur ne peut rien faire et doit comprendre pourquoi ;
- **la sonde nom (AC1) reste sur le défaut `includeArchived: false`** : un contact archivé n'est pas une fiche qu'on veut reprendre, et le remonter serait du bruit pur.

Écrire les deux sondes avec le même drapeau — dans un sens ou dans l'autre — casse l'une des deux.

## Acceptance Criteria

**AC1 — La frappe du nom fait apparaître les contacts proches.**
À partir de **trois caractères** saisis dans le champ de nom du formulaire de création (D7), les contacts **actifs** de la société dont le nom est proche sont proposés — archivés exclus (D12) —, avec de quoi les reconnaître : **nom complet, localité, et numéro de client** s'il existe (D11).
⚠️ Vaut pour les **deux** types de contact — raison sociale pour `Entreprise`, prénom + nom pour `Personne` (D8).
*Preuve*, trois tests de composant :
1. un sur `Entreprise` ;
2. un sur `Personne` — le seul qui tombe sous la mutation « ne lire que `formName` », et c'est sa raison d'être ;
3. **un contact ARCHIVÉ au nom proche n'apparaît PAS** dans les propositions. Symétrique de la preuve 2 d'AC2, et pour le même motif : les deux sondes ne diffèrent que par un booléen, écrit à deux endroits voisins (§ *Forme exacte des deux appels*). C'est le genre de permutation qu'aucun test ne rattrape si elle se produit dans l'autre sens.

**AC2 — Un numéro IDE déjà pris est signalé franchement, y compris quand le porteur est archivé.**
La saisie d'un IDE **complet** (`validateIdeFormat` vrai) déjà porté par un contact de la société — **actif ou archivé** (D12) — déclenche un avertissement explicite, **distinct** du signal « nom proche », **avant** l'enregistrement. Il n'est émis que si un contact remonté porte effectivement cet IDE (D10). Quand le porteur est archivé, le message **le dit**.
⚠️ La contrainte `uq_contacts_company_ide` refuse déjà le doublon **à l'enregistrement**, en `409 IDE_ALREADY_EXISTS` (`map_contact_error:536-541`). Cette AC ne remplace pas la garde : elle évite d'y arriver, et surtout d'y arriver **après** avoir saisi toute la fiche.
*Preuve*, **cinq** assertions distinctes — quatre tests de composant (portés par T4) et une vérification de bout en bout (la 4, portée par T6) :
1. un test de composant pour l'avertissement sur un porteur **actif** ;
2. **un test sur un porteur ARCHIVÉ** — c'est celui qui tombe sous la mutation `includeArchived: true → false`, et le seul qui couvre le cas sans recours de D12 ;
3. un test sur le cas de D10 — un contact qui remonte sans porter cet IDE ne déclenche **pas** le franc avertissement ;
4. un test qui vérifie que le `409` reste levé si l'avertissement est ignoré ;
5. **ouvrir en édition la fiche d'un contact qui porte un IDE ne déclenche AUCUN avertissement franc** — ni à l'ouverture, ni après avoir retapé le même IDE. C'est la preuve de la seconde condition de D10, et elle tombe sous la mutation « retirer `&& c.id !== editing?.id` ».

**AC3 — L'avertissement ne bloque jamais — l'interface, s'entend.**
Quel que soit le signal, franc ou nuancé, **l'interface laisse soumettre** : le bouton d'enregistrement reste actif, `formValidation` n'est pas touchée, la requête part.
⚠️ **Ce que devient la requête ensuite ne relève PAS de cette AC**, et la distinction n'est pas une subtilité de rédaction — sans elle, AC3 et la preuve 4 d'AC2 se contredisent formellement. Sur le signal **franc**, le contact ne peut **pas** être créé : le porteur de l'IDE existe, donc `uq_contacts_company_ide` lève un `409` — c'est précisément ce qu'exige la preuve 4 d'AC2. Écrire « ignorer l'avertissement crée bien le contact » sans distinguer les deux signaux prescrit un test E2E **impossible à écrire** pour le cas franc.
*Preuve* : test E2E sur le signal **nuancé** (nom proche) — ignorer l'avertissement crée bien le contact. ⚠️ **C'est l'assertion qui protège D1** : sa mutation — désactiver le bouton d'enregistrement — doit la faire tomber. Le cas franc est couvert par la preuve 4 d'AC2, qui documente le refus attendu.

**AC4 — Rien ne part avant que la frappe se calme, et une réponse tardive ne dit plus rien.**
La recherche est temporisée (300 ms, le pas du dépôt) et gardée par un compteur de génération : taper vingt caractères ne déclenche pas vingt requêtes, et une réponse arrivée en retard n'écrase pas un affichage plus récent.
Les deux sondes ont chacune **leur** minuterie et **leur** compteur (D13), et leur état repart de zéro à chaque ouverture du formulaire (D14).
*Preuve*, **cinq** tests :
1. le **nombre d'appels** pour une frappe continue sur le champ de nom ;
2. deux requêtes résolues **dans l'ordre inverse** de leur lancement — la première arrivée en dernier est ignorée ;
3. **un test croisé** : armer la sonde nom, puis la sonde IDE avant résolution, et vérifier que **les deux** avertissements aboutissent. Il tombe sous la mutation « factoriser une seule paire `(timer, compteur)` », que les deux autres tests laissent verte ;
4. **le nombre d'appels de la sonde IDE sur une suite valide → invalide → valide** (`CHE109322551` → on efface un chiffre → on le retape). Attendu : **deux** appels, un par passage à la validité — pas un de plus au passage par l'invalidité (D10), pas un de moins au retour. Sans lui, une sonde branchée sur chaque frappe du champ IDE passe les tests 1 à 3 ;
5. **la remise à zéro à l'ouverture** (D14) : armer une sonde, fermer le formulaire, le rouvrir — aucune proposition n'est affichée, et la réponse de la sonde armée avant la fermeture n'en fait pas apparaître. Il tombe sous la mutation « ne rien réinitialiser dans `openCreate()` », que rien d'autre n'attrape.
⚠️ Ne PAS prouver l'annulation par un `AbortSignal` passé à `apiClient` : il est écrasé en silence (D6), un tel test mesurerait du vide.

**AC5 — Le dispositif est muet quand il n'a rien à dire.** Deux clauses **distinctes**, qui n'ont pas la même conséquence :

- **(a) moins de trois caractères ⇒ AUCUNE requête** n'est émise (D7) ;
- **(b) trois caractères ou plus, aucun contact proche ⇒ aucun encombrement de l'écran, aucun message ;**
- **(c) une sonde qui ÉCHOUE est muette elle aussi** — réseau coupé, `500`, réponse illisible : aucun message d'erreur, aucune exception remontée, et **l'autre sonde continue de fonctionner**. Une sonde n'est pas une action de l'utilisateur : il n'a rien demandé, on ne lui doit donc aucun rapport d'échec.

⚠️ La conséquence « aucune requête » n'appartient qu'à (a). L'attacher aussi à (b) serait **impossible à satisfaire** : constater qu'aucun contact n'est proche suppose précisément d'avoir cherché.
⚠️ **Le seuil vaut dans les DEUX SENS.** Repasser sous trois caractères — effacer une lettre après en avoir tapé trois — remet le dispositif au silence : les propositions déjà affichées **disparaissent**, et la réponse d'une requête lancée à trois caractères qui arrive après la suppression ne les fait pas revenir (c'est le compteur de génération de D13 qui s'en charge). Un seuil qui ne joue qu'à la montée laisse un affichage figé sur un terme que l'utilisateur a déjà effacé.
*Preuve*, **quatre** tests : pour (a), **deux** — la saisie de deux caractères ne déclenche aucun appel réseau, **et** le retour de trois à deux caractères vide les propositions ; pour (b), **un** test de composant sur un carnet vide vérifiant qu'aucun message n'est rendu ; pour (c), **un** test où la sonde IDE répond `500` pendant que la sonde nom répond `200` — aucun message d'erreur n'est rendu, **et** les propositions de la sonde nom s'affichent normalement. Ce dernier tombe sous la mutation « retirer le `try/catch` d'une sonde », qu'aucun autre test n'attrape.

**AC6 — Les avertissements sont traduits dans les quatre locales.**
*Preuve* : un test Rust dédié dans `crates/kesh-i18n/src/loader.rs`, sur le patron exact de `client_number_labels_are_translated_in_all_four_locales` (`loader.rs:260-277`) — pour chaque clé neuve, `!= key` **et** `!= fr` sur `de-CH`, `it-CH`, `en-CH`. Plus `npm run lint-i18n-ownership` vert.
⚠️ **Il n'existe AUCUN test de parité globale dans `kesh-i18n`** — la note du 12 août parlait d'un « test d'appariement positionnel » qui n'existe pas. Le loader **replie silencieusement sur le français** (`format_missing_key_in_de_falls_back_to_fr`, `loader.rs:227`), et les fichiers sont déjà désappariés de 57 clés (KF #283). Une clé oubliée dans trois locales **ne rougit nulle part** : c'est l'assertion `!= fr` qui l'attrape, et rien d'autre.
⚠️ Le domaine `contact-*` compte aujourd'hui **62 clés dans chacune** des quatre locales — recompté le 2026-08-17, sans dérive. Le laisser apparié.

**AC7 — L'avertissement vaut aussi à l'édition, sans se signaler lui-même.**
Renommer un contact existant vers un nom déjà porté déclenche le même signal nuancé. ⚠️ Le contact **en cours d'édition est exclu** de ses propres résultats : sans cette garde, ouvrir une fiche et toucher son nom afficherait « un contact au nom proche existe » en désignant **la fiche elle-même** — un avertissement absurde, qui est la façon la plus rapide d'apprendre à l'utilisateur à ne plus les lire.
⚠️ **Cette AC ne couvre que la sonde NOM.** L'exclusion de soi sur la sonde **IDE** est portée par D10 et prouvée par la preuve 5 d'AC2 — elle est **plus critique encore**, puisqu'elle touche le signal franc et se déclenche sur la simple ouverture d'une fiche pourvue d'un IDE. Les deux sondes ne partagent pas leur code : la garde doit être écrite **deux fois**, et prouvée **deux fois**.
*Preuve* : test de composant — ouvrir la fiche d'un contact `Entreprise`, modifier sa raison sociale d'un caractère, vérifier qu'il ne figure pas dans les propositions.
Un seul type suffit **ici**, et la raison se vérifie : la garde est `c.id !== editing?.id`, qui ne consulte **ni** le type **ni** le terme. Ce qui dépend du type, c'est la **composition** du terme (D8) — couverte par la preuve 2 d'AC1 et les cas limites de T2, pas par une duplication de ce test.

## Tasks / Subtasks

- [ ] **T1 — Rendre l'IDE cherchable** (AC2). Ajouter `ide_number` aux **DEUX** branches `LIKE` de `push_where_clauses` (`crates/kesh-db/src/repositories/contacts.rs:195-210`), et **seulement** cela — aucune route neuve (D5).
  - [ ] ⚠️ **Les deux branches, ou aucune** (D3). N'en traiter qu'une compile, passe les tests dont le terme survit à `escape_boolean_ft`, et cesse **silencieusement** de chercher l'IDE quand le terme n'est fait que d'opérateurs FULLTEXT.
  - [ ] Deux tests `#[sqlx::test]`, **un par branche** : le cas courant (`search=CHE109322551`) et le cas où `escaped.is_empty()`, c'est-à-dire un terme **intégralement** composé des 10 opérateurs `+ - > < ( ) ~ * " \` — par exemple `search=***`.
  - [ ] ⚠️ **Ne PAS se caler sur `test_search_handles_special_chars:1283` pour la seconde branche : il ne l'exerce pas.** Il cherche `"100%"`, et `%` **n'est pas** un opérateur BOOLEAN MODE (`crates/kesh-db/src/util/search.rs:41`) — le terme survit donc intact à `escape_boolean_ft`, et le test emprunte la branche `else`. Vérifié en passe 1 de validate. **Aucun test du dépôt n'exerce aujourd'hui la branche `escaped.is_empty()`** : il n'y a pas de patron à copier, seulement un terme à choisir correctement.
  - [ ] ⚠️ **`push_where_clauses` est un chemin PARTAGÉ** — cf. § *Rayon d'impact de T1*. Relancer `binary(contacts)` **et** les suites qui consomment la liste de contacts, pas seulement le test qu'on vient d'écrire.
  - [ ] **Conditionné à l'arbitrage Q1.** Si Q1 est tranchée « ne pas élargir la recherche du carnet », T1 devient une route dédiée et AC2 change de coût — **demander avant d'implémenter**.
- [ ] **T2 — Logique pure de proposition** (AC1, AC2, AC5, AC7). Un module `.ts` sans clé i18n ni DOM (D9) : seuil de 3 caractères **dans les deux sens** (AC5), choix du terme selon le type (D8), exclusion du contact édité sur **les deux** sondes (AC7 et D10), vérification de l'IDE sur le champ (D10).
  - [ ] Tests vitest directs, sans `render` — c'est le bénéfice de D9.
  - [ ] **Cas limites de composition du terme** (D8), à couvrir explicitement : `Personne` avec le nom encore vide (`"Jean"`), avec le prénom encore vide (`"Dupont"`), avec les deux vides (`""` ⇒ sous le seuil, donc muet), et la **bascule de type en cours de frappe** `Entreprise ⇄ Personne`. Le formulaire n'exige prénom **et** nom qu'à la soumission (`validate_common:362-370`) : pendant la frappe, l'un des deux est presque toujours vide, et c'est l'état **normal** de la sonde, pas un cas dégradé.
- [ ] **T3 — Temporisation et garde d'ordre** (AC4). Débounce 300 ms + compteur de génération, sur le patron de `ContactPicker.svelte:36-78` (D6). **Une paire `(timer, compteur)` PAR SONDE** (D13) — `ContactPicker` n'a qu'un flux et ne peut pas servir de modèle sur ce point.
  - [ ] Réutiliser `debounce` de `$lib/features/journal-entries/debounce.ts` **ou** le `setTimeout` inline déjà présent dans `+page.svelte:185-192` — ne pas écrire un troisième mécanisme. ⚠️ Le helper est mal rangé (dossier `journal-entries` pour un utilitaire général) : le **déplacer** est hors périmètre, s'en servir ne l'est pas.
  - [ ] Remise à zéro des deux paires dans `openCreate()` **et** `openEdit()` (D14) — **pas** sur la fermeture du dialogue, qui a trois sites (`:352`, `:358`, `:846`) et n'en aura pas moins demain.
  - [ ] Nettoyer les timers au démontage — `+page.svelte:113` le fait déjà pour la recherche de liste, les nouveaux doivent l'être aussi.
  - [ ] Brancher les sondes sur **`oninput`**, comme `ContactPicker.svelte:127` et `+page.svelte:463`, et **jamais sur `onkeydown`** — un collage ne produit aucune frappe clavier, et une sonde branchée sur le clavier resterait muette sur le geste le plus courant de tous : coller un nom depuis un courriel. (`ContactPicker` a bien un `onkeydown:133`, mais il ne pilote que la navigation au clavier dans la liste.)
  - [ ] **Écrire les 5 tests de la preuve d'AC4** — cf. le § *Décompte des preuves*.
- [ ] **T4 — Balisage et signaux** (AC1, AC2, AC3, AC5, AC7). Deux niveaux visuellement distincts (D2), aucun blocage : ne toucher **ni** au `disabled` du bouton d'enregistrement **ni** à `formValidation` (`+page.svelte:275`).
  - [ ] Il n'existe pas de composant `alert` dans `lib/components/ui/` — se caler sur le style d'erreur déjà employé par le formulaire (`formError`), sans créer de primitive nouvelle.
  - [ ] Encapsuler **chaque** sonde dans un `try/catch` qui se tait, comme `ContactPicker.svelte:64-67`. Une sonde n'est pas une action de l'utilisateur : son échec ne doit produire ni message d'erreur ni exception remontée.
  - [ ] **Créer `frontend/src/routes/(app)/contacts/+page.test.ts`** — il n'existe **aucun** test de cette page aujourd'hui (seul `contact-helpers.test.ts` existe pour ce domaine). Patron : `products-page.test.ts`, mocks hoistés avant l'import. Il porte les **12 tests de composant** du décompte ci-dessous.
  - [ ] ⚠️ **Cette sous-tâche est la raison d'être de T4.** Sans elle, T4 se coche en ayant livré du balisage, et **aucune** des preuves d'AC1, AC2, AC5 et AC7 n'existe — relevé en passe 1 de validate.
- [ ] **T5 — i18n** (AC6). Les clés neuves dans les **quatre** FTL (`crates/kesh-i18n/locales/{fr,de,en,it}-CH/messages.ftl`), domaine `contact-*`, plus le test Rust dédié.
- [ ] **T6 — E2E** (AC3, **et la preuve 4 d'AC2**). ⚠️ Le fichier **DOIT** être nommé `*.spec.ts` : `playwright.config.ts:35` filtre sur `testMatch: /(.+\.)?spec\.[jt]s/`, et un `*.test.ts` posé dans `tests/e2e/` est **silencieusement ignoré** — il ne rougit jamais, il se tait.
  - [ ] Patron : `frontend/tests/e2e/contact-client-number.spec.ts` (Story 22-1) — `seedTestState('with-company')`, login, et **archivage du contact créé** en fin de test pour ne polluer ni l'unicité ni les tests suivants.
  - [ ] **Assertion 1 (AC3)** : ignorer le signal **nuancé** (nom proche) et soumettre **crée bien le contact** — vérifié après rechargement.
  - [ ] **Assertion 2 (preuve 4 d'AC2)** : ignorer le signal **franc** (IDE déjà pris) et soumettre **échoue en `409`**, et l'interface le dit. C'est la seule preuve de la story qui traverse réellement la frontière HTTP jusqu'à la contrainte.
  - [ ] ⚠️ **Sans ces deux lignes, T6 se coche sur un fichier E2E qui monte le décor et n'assert rien** — un test vide passe, et il se tait exactement comme le ferait un fichier mal nommé. C'est le même mode d'échec que celui contre lequel la ligne du `testMatch` met en garde, un cran plus loin. Relevé en passe 2 de validate.
- [ ] **T7 — Documentation** (AC1, AC2). `docs/manual/fr/user-manual.tex`, § *Contacts* → *Création d'un contact* (ligne 506) : ce que l'avertissement dit, et **qu'il n'empêche rien**. Régénérer le PDF (`make fr` dans `docs/manual/`) et le commiter — la convention du dépôt est de versionner les PDF.
  - [ ] ⚠️ Ne PAS toucher à la ligne 538 (« *Contacts → Import CSV* »), qui annonce une fonction inexistante : c'est **#291**, une issue ouverte à part. La corriger ici mélangerait deux sujets dans une PR.

## Décompte des preuves — la seule table qui fait foi

⚠️ **Ce tableau est l'unique compteur de la story.** Aucun autre passage ne doit énoncer un total : trois compteurs épars avaient déjà dérivé en une seule passe de revue (le titre d'AC2 disait « quatre » pour cinq preuves, celui des pièges muets « six » pour sept, un Change Log « 8 tests » pour dix). Un nombre qui n'a qu'un seul site ne peut pas se désynchroniser. Il se **recompte depuis les AC**, jamais ne s'incrémente de confiance.

| AC | Preuves | Nature | Portées par |
|---|---:|---|---|
| AC1 — nom proche | 3 | composant | T4 |
| AC2 — IDE déjà pris | 4 + 1 | composant + E2E | T4 · T6 |
| AC3 — ne bloque jamais | 1 | E2E | T6 |
| AC4 — temporisation et ordre | 5 | unitaire | T3 |
| AC5 — muet quand rien à dire | 4 | composant | T4 |
| AC6 — quatre locales | 1 | unitaire Rust | T5 |
| AC7 — édition sans se signaler | 1 | composant | T4 |
| *(T1 — l'IDE cherchable)* | 2 | `#[sqlx::test]` | T1 |

**Totaux, sommés depuis la colonne** : **12 tests de composant** (3 + 4 + 4 + 1) · **5 tests unitaires** front · **2 assertions E2E** · **1 test unitaire Rust** i18n · **2 tests d'intégration** base. **Soit 22 preuves.**

Réserve honnête sur une seule ligne : la première preuve de la clause (a) d'AC5 (« moins de trois caractères ⇒ aucun appel ») peut légitimement vivre dans le module pur de T2 plutôt que dans le fichier de page. Elle reste comptée ici — ce qui n'est pas négociable, c'est **qu'elle existe**, pas l'endroit où elle vit.

## Dev Notes

### Ce qui existe déjà, et qu'il ne faut pas réinventer

**La recherche.** `GET /api/v1/contacts` (`crates/kesh-api/src/routes/contacts.rs:563-603`) est scopé par `current_user.company_id`, borne le `limit` à 100 (`MAX_LIST_LIMIT`), et délègue à `contacts::list_by_company_paginated`. `push_where_clauses` (`crates/kesh-db/src/repositories/contacts.rs:150-213`) combine un index **FULLTEXT** sur `name` et un `LIKE` sur l'email et le numéro de client. Le commentaire du fichier explique pourquoi le numéro de client n'est **pas** dans l'index FULLTEXT — ses séparateurs cassent les tokens. **Le même raisonnement vaut pour l'IDE** (`CHE-123.456.789`), d'où le `LIKE` de T1.

**Le typeahead.** `frontend/src/lib/components/invoices/ContactPicker.svelte` fait déjà, pour la sélection d'un contact sur une facture, l'essentiel de ce que cette story demande pour la saisie : débounce 300 ms, compteur `searchSeq`, nettoyage au démontage, pattern ARIA combobox. **C'est le précédent à lire avant d'écrire une ligne.**
⚠️ Y noter aussi ce qu'il ne faut **pas** copier : ses libellés sont en français en dur, non traduits (« Chargement… », « Aucun contact »). AC6 ne le tolère pas pour les nôtres.
⚠️ Et ce qu'il faut copier sans discuter : `$props.id()` et **jamais** `crypto.randomUUID()` — cette API n'existe qu'en contexte sécurisé et crashe le rendu sur un déploiement HTTP LAN comme le NAS (#145).

**Les gardes d'unicité déjà en place.** `uq_contacts_company_ide` (depuis `20260414000001_contacts.sql:23`) et `uq_contacts_company_client_number` (16-3b) refusent le doublon à l'enregistrement, remappés en `409` par `map_contact_error` (`routes/contacts.rs:536-555`). **Le nom n'a aucune contrainte d'unicité, et n'en aura pas** — c'est D1.

**Les helpers IDE.** `contact-helpers.ts` : `validateIdeFormat` (format seul, pas le checksum), `normalizeIdeForApi` (`CHE-123.456.789` → `CHE123456789`), `formatIdeNumber` (l'inverse). Le checksum n'est validé qu'au backend, par `CheNumber::new` (`routes/contacts.rs:312-322`) — un IDE au bon format mais au mauvais checksum passe donc la porte de D10 et sera refusé en `400` à l'enregistrement. C'est acceptable : on cherche un doublon, pas on ne valide un IDE.

### Rayon d'impact de T1 — un chemin partagé, pas un chemin à nous

⚠️ `push_where_clauses` n'est pas privé de cette story. Il sert **tout** ce qui liste des contacts, et T1 change donc le comportement de trois consommateurs que la story ne touche pas :

| Consommateur | Effet de l'ajout d'`ide_number` |
|---|---|
| La recherche du carnet (`+page.svelte:462`) | chercher un IDE y remonte désormais son contact — **c'est l'objet de Q1** |
| `ContactPicker.svelte` (choix du débiteur sur une facture) | même élargissement, non demandé, non nuisible |
| Tout test existant qui compte des résultats de recherche | un terme qui matche aussi un IDE peut changer un `total` |

Les tests à surveiller nommément : `test_filter_by_search_name:1225`, `test_search_handles_special_chars:1283`, `test_search_no_longer_matches_mid_word:1328`. Aucun ne devrait bouger — les IDE des seeds ne contiennent pas les termes qu'ils cherchent — mais **c'est à vérifier par exécution, pas par lecture**.

Le geste correct est celui de la § *Pendant une boucle de revue* du `CLAUDE.md` : gate ciblé `binary(contacts)` entre les passes, gate **complet** au push. Et comme le patch touche `crates/kesh-db/`, l'exception qui interdit le ciblage s'applique **au dernier commit** : gate complet obligatoire.

### Forme exacte des deux appels

Pour qu'aucun paramètre ne se perde entre l'intention et le code :

```ts
// AC1 — sonde « nom proche ». Actifs SEULEMENT (D12), soi-même exclu (AC7).
const rn = await listContacts({ search: term, limit: 5, includeArchived: false });
const proches = rn.items.filter((c) => c.id !== editing?.id);

// AC2 — sonde IDE. Archivés COMPRIS (D12), soi-même exclu (D10),
// puis vérification SUR LE CHAMP — un résultat n'est pas une preuve (D10).
const ri = await listContacts({ search: normalized, limit: 5, includeArchived: true });
const holder = ri.items.find((c) => c.ideNumber === normalized && c.id !== editing?.id);
```

⚠️ **Les deux appels ne diffèrent que par un booléen et se lisent côte à côte.** C'est délibérément le passage le plus dangereux de la spec : une inversion de `includeArchived` ne casse rien, ne fait rougir aucun test qui ne la vise pas, et retourne des résultats plausibles dans les deux sens. D'où la preuve 3 d'AC1 **et** la preuve 2 d'AC2 — une par sens.

`limit` est borné à 100 par `MAX_LIST_LIMIT` (`routes/contacts.rs:42`) ; 5 suffit largement et tient l'affichage court, ce qui sert directement le « piège de cette story » ci-dessous. `ContactResponse` porte `ideNumber`, `clientNumber` et `addressStructured` (donc la localité) — **rien à ajouter au DTO**.

### Le piège de cette story

Un avertissement trop bavard **ne protège plus rien** : on apprend à le fermer sans le lire, et il devient un clic de plus. C'est le vrai risque, plus que le faux négatif. Le seuil doit donc être **serré** — mieux vaut manquer un doublon que crier trois fois par jour à tort. AC7 (ne pas se signaler soi-même) et D10 (ne pas crier « IDE pris » à faux) ne sont pas des raffinements : ce sont les deux façons les plus rapides de rendre le dispositif inaudible dès la première semaine.

### Les sept pièges muets, nommés

Aucun des sept ne casse la compilation, ne fait rougir un test, ni ne produit d'erreur au runtime. C'est ce qui les rend coûteux — et c'est pourquoi chacun a sa preuve dédiée, dont la **mutation** est écrite en face.

| Piège | Symptôme | Garde |
|---|---|---|
| `signal` passé à `apiClient` (D6) | l'annulation ne fait rien, on croit l'avoir | AC4 prouvée par ordre de résolution, jamais par `AbortSignal` |
| déclencheur sur `#form-name` seul (D8) | inopérant pour toutes les personnes physiques | AC1 preuve 2, sur `Personne` |
| sonde IDE sur les actifs seuls (D12) | silence, puis `409` sans coupable visible | AC2 preuve 2, sur un porteur **archivé** |
| sonde nom sur les archivés (D12, sens inverse) | propositions polluées par des fiches mortes | AC1 preuve 3 |
| sonde IDE sans exclusion de soi (D10) | le signal **franc** crie à faux à chaque édition | AC2 preuve 5 |
| une seule paire `(timer, compteur)` (D13) | l'un des deux avertissements ne s'affiche jamais | AC4 preuve 3, le test croisé |
| clé i18n absente de 3 locales (AC6) | repli silencieux sur le français | assertion `!= fr`, la seule qui l'attrape |

### Ce que la story ne répare pas, et qu'elle rend plus atteignable

⚠️ **Une session qui expire pendant la composition d'une fiche fait perdre la saisie.** Tout `401` — y compris sur un `GET` de fond — déclenche un refresh, et si le refresh échoue, `api-client.ts:78-79,88-98,157-158` fait `authState.clearSession()` puis `window.location.replace('/login?reason=session_expired')` : une redirection **plein document**, pas une navigation SPA. Le formulaire, non enregistré, disparaît.

C'est un comportement **préexistant** de l'`apiClient`, et cette story ne le corrige pas — mais elle le rend **plus probable d'être atteint** : jusqu'ici il fallait cliquer « Enregistrer » pour émettre une requête depuis ce formulaire, désormais des sondes partent toutes les 300 ms pendant la frappe. Le risque est **accepté explicitement**, pas ignoré ; le `try/catch` de T4 protège du reste (réseau, `500`), mais pas de celui-là, qui se produit dans l'`apiClient` avant que l'appelant ne reprenne la main. Si le sujet doit être traité, c'est une issue à part.

### Previous story intelligence — Story 22-1

Elle a touché **le même fichier** (`repositories/contacts.rs`, +271 lignes) et **la même table**. Ce qu'elle laisse et qui sert ici :

- `kesh_core::text::canonical_key` et `is_invisible` — l'unique source du repli de casse et du retrait des invisibles. Si un jour on veut rapprocher deux **noms** modulo la casse, c'est là qu'il faut aller chercher, pas réécrire. *(Cette story n'en a pas besoin : le FULLTEXT est déjà insensible à la casse.)*
- `normalize_optional` (`routes/contacts.rs:299-308`) : `""` et les valeurs intégralement invisibles s'effondrent en `None` **avant** toute chose. Un champ « vide à l'écran » n'est jamais stocké.
- La leçon de méthode, écrite dans son propre change log : *une hypothèse éliminée par raisonnement n'est pas une hypothèse testée.* D6 et D7 de cette story ont été **exécutés**, pas déduits — la variable MariaDB a été lue sur la base de dev, le spread de `fetchWithTimeout` relu à la ligne.
- ⚠️ **Elle a bumpé `kesh_version_min_required` à `0.10.0`** (migration `20260814000001`). Cette story **n'introduit aucune migration** — donc aucun bump, ni Cargo ni `min_required`. Si une migration devait apparaître, relire la § *Migration breaking policy* du `CLAUDE.md` en entier, P2-bis compris.

### Git intelligence

Les cinq derniers commits de `main` sont l'Epic 22 en cours : `70e6a2d0` (plafond nextest 6 → 8), `5dd4b091` (bookkeeping 22-5), `94b5643e` (squash du schéma de test), `8b3a99d0` (22-1), `01e59a20` (22-4). Deux conséquences pratiques :

- **La suite de tests coûte désormais ~1,5 min** en local (tmpfs + squash), contre plus d'une heure avant la 22-5. Le gate complet n'est plus une raison de tricher.
- ⚠️ **`.config/nextest.toml` est à 8 threads depuis le 2026-08-16, et son innocuité n'est PAS établie** (trois runs propres, contre un défaut qui ne se montrait qu'un run sur trois). Si un `reconciliation_*_e2e` rougit sans rapport avec cette story, **commencer par là** — et remettre la base à zéro avant de diagnostiquer quoi que ce soit (§ *Un gate laisse la base piégée*, KF-039 #310).

### Conventions de test

**Backend.** `#[sqlx::test]` monte le squash `crates/kesh-db/test-schema/` — ne pas ajouter d'attribut sur le vrai `MIGRATOR` sans lire `crates/kesh-db/tests/test_schema_guard.rs`, qui tient la liste des 43 exceptions.

**Frontend.** `@testing-library/svelte` v5 + Svelte 5. Patron de page complet : `frontend/src/routes/(app)/products/products-page.test.ts` — mocks hoistés **avant** l'import du composant, `$app/environment`, `$app/navigation`, `$app/state`, `i18n.svelte` et l'API mockés. Le doc-comment de ce fichier vaut d'être lu : il explique pourquoi une assertion sur **l'argument** d'un mock attrape des mutations qu'un test fonctionnel laisse passer.

**Mutations jouées, pas raisonnées.** Pour AC3, la mutation est explicite : désactiver le bouton d'enregistrement doit faire tomber le test. Pour AC1, la mutation est « ne lire que `formName` » et c'est le test `Personne` qui tombe.

**Les affirmations d'absence se vérifient au `grep -nF`** avant d'être écrites — et le grep porte sur la **valeur** (`\b3\b`, `ide_number`), jamais sur la phrase qui l'entoure.

### Project Structure Notes

- **Backend** : un seul fichier touché si Q1 va dans le sens de D5(a) — `crates/kesh-db/src/repositories/contacts.rs`. Aucune route neuve, aucun DTO neuf, **aucune migration**.
- **Frontend** : la logique pure dans un `.ts` (D9), le balisage dans `frontend/src/routes/(app)/contacts/+page.svelte` — qui fait déjà 900 lignes et porte le formulaire en `Dialog` (ligne 633). C'est le seul site : il n'existe **pas** de second formulaire de contact.
- **i18n** : les quatre FTL de `crates/kesh-i18n/locales/`. Le frontend les consomme via `GET /api/v1/i18n/messages` (`i18n.svelte.ts`), donc **une seule source** pour les deux côtés.
- **Variance assumée** : le helper `debounce` reste dans `features/journal-entries/`, où il est mal rangé. Le déplacer toucherait `journal-entries/+page.svelte` sans rapport avec cette story — noté, pas traité.

### References

- Issue **#301** — le besoin, et les trois points à instruire (sur quoi apparier, le seuil, le coût de la requête).
- Story **22-1** (`22-1-unicite-canonique-numero-client.md`) — `canonical_key`, la colonne canonique, et la leçon de méthode.
- Story **22-3** (#300) — la fusion, en veille : ce que cette story doit rendre inutile.
- Issue **#302** — la succession d'entreprise, à ne pas confondre.
- Issue **#291** — le manuel annonce un import CSV inexistant, dans la section que T7 touche. **À ne pas traiter ici.**
- Story **7-4 / KF-005** (`7-4-kf-005-fulltext-search-index.md`) — l'analyse d'origine du FULLTEXT, dont le § *UX impact tokens courts* et le tableau `innodb_ft_min_token_size` vs `ft_min_word_len`.
- `CLAUDE.md` — § *Un appariement automatique propose, il ne crée jamais*, § *Test Locally First*, § *Un gate laisse la base piégée*.

## Questions ouvertes

**Q1 — Élargir la recherche du carnet à l'IDE, ou lui réserver un chemin à part ?** *(bloquante pour T1)*
D5(a) ajoute `ide_number` aux deux branches `LIKE` de la recherche existante : un seul chemin, une seule garde de scoping, deux tests. **Conséquence visible** : chercher `CHE-109.322.551` dans le carnet remontera désormais son contact — ce qui paraît un gain, cohérent avec ce que la 16-3b a fait du numéro de client, mais qui change un comportement que personne n'a demandé de changer.
D5(b) écrit une route dédiée, sans toucher à la recherche — au prix d'un second chemin à scoper, ce que T1 identifie comme le risque principal.
**Recommandation : (a).** Mais c'est un changement d'UX, donc l'arbitrage revient à Guy.

**Q2 — AC7 (l'édition) est-elle dans le périmètre de cette story ?**
La note du 12 août posait la question et répondait « probablement oui, et à moindres frais puisque le dispositif sera écrit ». C'est exact — le surcoût est une exclusion par `id` et un test. Elle est donc **écrite comme AC7**, mais elle reste la seule AC détachable si le périmètre doit être resserré. La détacher ne laisse **aucun trou fonctionnel** : le formulaire d'édition se comporterait comme aujourd'hui.

## Change Log

### Passe 1 de `bmad-create-story validate` — 2026-08-17, Sonnet ×3, contexte frais

Trois lentilles orthogonales (BlindHunter, EdgeCaseHunter, AcceptanceAuditor), lancées en parallèle sur contexte frais, ground-truth obligatoire. Bruts : 1 + 5 + 4 = 10 findings. **Après dédoublonnage : 1 CRITICAL / 2 HIGH / 6 MEDIUM / 0 LOW — 9 findings > LOW, 9 correctifs appliqués, 0 écarté, 0 faux positif.**

**Le finding du jour est une CONVERGENCE de deux lentilles indépendantes** : AA-1 (`CRITICAL`) et ECH-1 (`HIGH`) décrivent le même défaut — **la sonde IDE n'excluait pas le contact en cours d'édition**. `openEdit` pré-remplit `formIde` avec l'IDE du contact lui-même (`+page.svelte:256`), si bien que l'avertissement **franc** — celui auquel on demande à l'utilisateur de faire confiance — se serait déclenché sur sa propre fiche, à l'ouverture, sur le chemin le plus banal du formulaire. J'avais écrit la garde pour la sonde nom (AC7) et **je ne l'avais pas propagée à la sonde IDE** : les deux sondes ne partagent pas leur code, et un raisonnement ne se propage pas tout seul. La sévérité `CRITICAL` de l'AcceptanceAuditor est retenue plutôt que le `HIGH` de l'EdgeCaseHunter — pas par prudence, mais parce que le défaut touche le signal fort et se déclenche **sans action de l'utilisateur**.

Les huit autres, par ordre de sévérité :

| # | Sév. | Défaut | Correctif |
|---|---|---|---|
| AA-2 | HIGH | AC3 (« ignorer l'avertissement crée bien le contact ») **contredit formellement** la preuve 4 d'AC2 (« le 409 reste levé ») — sur le signal franc, le contact ne PEUT pas être créé. Le test E2E prescrit était impossible à écrire. | AC3 recentrée sur *l'interface ne bloque pas* ; la preuve E2E est restreinte au signal nuancé |
| AA-3 | HIGH | T3 et T4 **ne portaient aucune sous-tâche de test** alors que T4 seule porte AC1, AC2, AC3, AC5 et AC7. Un dev cochait T4 après le seul balisage, sans qu'aucune preuve existe — et **aucun test de cette page n'existe** dans le dépôt (`find` → seul `contact-helpers.test.ts`). | sous-tâches de test ajoutées à T3 et T4, fichier `+page.test.ts` nommé ; le décompte annoncé ici (« 8 ») était **faux dès son écriture** et a été refait en passe 2 — cf. § *Décompte des preuves* |
| BH-1 | MED | Le patron cité pour la branche `escaped.is_empty()` **ne l'exerce pas** : `test_search_handles_special_chars` cherche `"100%"`, et `%` n'est pas un opérateur BOOLEAN MODE (`util/search.rs:41`) — le terme survit et prend la branche `else`. **Aucun test du dépôt n'exerce cette branche.** | terme concret prescrit (`***`), et la fausse piste explicitement démontée |
| ECH-3 | MED | L'état des sondes n'était remis à zéro nulle part : le dialogue se ferme depuis **3 sites** et n'est jamais démonté. Une réponse armée avant « Annuler » s'affichait sur la fiche suivante. | **D14** — remise à zéro dans `openCreate()`/`openEdit()`, pas sur la fermeture |
| ECH-4 | MED | Rien n'imposait **deux** paires `(timer, compteur)`. Une paire factorisée fait qu'une réponse tardive de l'une invalide la fraîcheur de l'autre — un des deux avertissements ne s'affiche jamais, en silence. | **D13** + preuve 3 d'AC4, le test croisé |
| ECH-5 | MED | AC1 affirmait « archivés exclus » **sans aucune preuve**, alors que les deux appels ne diffèrent que par ce booléen et se lisent côte à côte. | preuve 3 d'AC1, symétrique de la preuve 2 d'AC2 |
| AA-4 | MED | AC5 attachait « aucune requête » à **deux** déclencheurs, dont l'un (« aucun contact proche ») ne peut être constaté qu'**après** avoir cherché — indécidable à la lettre. | AC5 scindée en clauses (a) et (b) |
| ECH-2 | MED | Une session expirée pendant la frappe redirige en **plein document** et efface la saisie ; les sondes rendent ce risque préexistant bien plus atteignable. | `try/catch` prescrit en T4 + risque **accepté explicitement** en Dev Notes |

**Ce que la passe n'a PAS trouvé, et qui compte autant** : aucune des dizaines de citations `fichier:ligne` de la spec n'était fausse — les trois lentilles les ont vérifiées indépendamment, y compris la mesure en direct de `innodb_ft_min_token_size` et l'asymétrie des deux contraintes d'unicité. Le ground-truth de la spec tient ; ce qui a cédé, c'est sa **couverture** — des raisonnements justes appliqués à une sonde et pas à l'autre.

⚠️ **Passe 2 requise** par la § *Review Iteration Rule* (1 CRITICAL + 2 HIGH). Rotation : **Haiku**, contexte frais — avec le garde-fou du `CLAUDE.md` § *Haiku-specific guardrails*, tout `CRITICAL`/`HIGH` affirmant une absence ou une présence sera ground-truthé au `grep -nF` avant d'être traité.

### Passe 2 de `bmad-create-story validate` — 2026-08-17, Haiku 4.5 ×3, contexte frais

Bruts : 3 + 8 + 3 = 14. Un doublon (BH2-2 ≡ AA2-2), une fusion (ECH2-6 + ECH2-7 traitent le même sujet), **un finding réfuté**. **Retenus : 0 CRITICAL / 1 HIGH / 9 MEDIUM / 1 LOW — 11 findings, 11 correctifs appliqués.**

**Trend : `1 CRIT / 2 HIGH / 6 MED` → `0 CRIT / 1 HIGH / 9 MED`.** La sévérité maximale **décroît** (CRITICAL → HIGH), donc le critère de non-convergence de la § *Règle de splitting préventif* n'est **pas** déclenché — mais il faut dire aussi ce qui monte : **les MEDIUM passent de 6 à 9**, et ce n'est pas du bruit.

**Le fil rouge de cette passe, et il est à ma charge : ce sont les correctifs de la passe 1 qui ont produit l'essentiel de ses findings.** Deux familles, toutes deux nommées dans le `CLAUDE.md` :

- **Trois décomptes faux, tous introduits par mes propres patches de passe 1** — le titre d'AC2 annonçait « quatre assertions » après que j'y en eus ajouté une cinquième ; celui des pièges muets « six » pour sept lignes ; le Change Log de la passe 1 « 8 tests » quand la tâche en dénombrait dix. C'est mot pour mot la § *Recompter ses propres comptes rendus* : **le compte rendu devient le lieu du défaut**. La réponse n'est pas de corriger trois nombres mais de **supprimer les compteurs épars** : un § *Décompte des preuves* unique fait désormais foi, et aucun autre passage n'énonce de total.
- **Quatre décisions ajoutées en passe 1 sans leur preuve** — D14 (remise à zéro à l'ouverture), le `try/catch` des sondes, le seuil dans le sens descendant, les transitions de validité de l'IDE. C'est la § *Propagation post-patch* : un patch qui pose une obligation sans poser le test qui la tient n'est pas terminé. Chacune a maintenant sa preuve **et sa mutation**.

Le seul `HIGH` relève de la même famille : **T6 ne prescrivait aucune assertion** — on pouvait cocher la tâche sur un fichier E2E qui monte le décor et n'affirme rien. C'est le mode d'échec du test muet, un cran au-delà de celui contre lequel la ligne du `testMatch` met en garde. T6 porte désormais ses deux assertions nommées, dont la preuve 4 d'AC2 (le `409`), qui n'était **rattachée à aucune tâche**.

Le reste, par famille : le seuil de trois caractères ne jouait qu'à la montée (AC5) ; les cas limites de composition du terme pour une `Personne` en cours de frappe n'étaient pas couverts, alors que l'un des deux champs est presque toujours vide à ce moment (T2) ; AC7 ne disait pas sur quel type de contact porte sa preuve — précisé, avec la raison pour laquelle **un seul type suffit ici** (la garde est `c.id !== editing?.id`, qui ne consulte ni le type ni le terme).

**Un finding RÉFUTÉ, au grep** : ECH2-8 affirmait qu'un collage échapperait à la temporisation faute d'événement clavier. Vérifié — `grep -nF "oninput"` : les deux champs de recherche du dépôt sont liés à **`oninput`** (`ContactPicker.svelte:127`, `+page.svelte:463`), qui se déclenche sur un collage ; le `onkeydown:133` de `ContactPicker` ne pilote que la navigation dans la liste. Le patron que la spec fait copier traite donc le cas par construction. Le finding est écarté — mais il a produit une garde utile : T3 dit maintenant explicitement de brancher sur `oninput` et **jamais** sur `onkeydown`, parce qu'un dev qui l'inverserait resterait muet sur le geste le plus courant de tous.

**Ce que la passe 2 n'a PAS trouvé** : aucune fausseté factuelle, aucune régression des correctifs de la passe 1, aucune citation `fichier:ligne` erronée — les trois lentilles Haiku ont re-vérifié indépendamment les ancres du code, et **aucune hallucination du mode d'échec documenté au `CLAUDE.md`** n'est survenue. Le mécanisme spécifié n'a été mis en défaut par aucune des six lentilles des deux passes ; ce qui cède, passe après passe, c'est la **tenue du document**.

⚠️ **Passe 3 requise** par la § *Review Iteration Rule* (1 HIGH > LOW). Rotation : **Opus**, contexte frais.

## Dev Agent Record

### Agent Model Used

Claude Opus 5 (1M context) — `bmad-create-story`, 2026-08-17.

### Debug Log References

Vérifications au sol menées pendant la rédaction (toutes **exécutées**, aucune déduite) :

- `innodb_ft_min_token_size = 3` — lu sur la base de dev, `SHOW VARIABLES`.
- `contact-*` = **62 clés** dans chacune des 4 locales — `grep -c '^contact-'`.
- Absence de test de parité globale dans `kesh-i18n` — `ls crates/kesh-i18n/tests/` (inexistant) + relecture de `loader.rs`, qui le dit lui-même en commentaire (ligne 289).
- Écrasement de `init.signal` — relu à la ligne, `api-client.ts:259-268`.
- `testMatch: /(.+\.)?spec\.[jt]s/` — `playwright.config.ts:35`.
- Piège du lint i18n singulier/pluriel — `lint-i18n-ownership.js:154-163` + `KNOWN_VIOLATIONS:99-111`.
- **Asymétrie des deux contraintes d'unicité** (D12) — les deux migrations relues (`20260414000001:23` plate, `20260810000001:25-49` partielle). L'hypothèse « les deux se comportent pareil à l'archivage » a été **lue dans le SQL, pas supposée** ; elle est fausse, et c'est ce qui a fait ajouter la deuxième preuve d'AC2.

### Completion Notes List

- Spec créée le 2026-08-17 à partir de la note d'analyse du 2026-08-12, dont **une affirmation a été réfutée** : le « test d'appariement positionnel de `kesh-i18n` » invoqué comme preuve d'AC6 **n'existe pas**. AC6 porte désormais la preuve réelle, et la raison pour laquelle son absence est dangereuse (repli silencieux sur le français, KF #283).
- Les deux questions ouvertes de la note ont été traitées : la première est **tranchée** (D11 — la localité s'affiche, ne se cherche pas), la seconde **écrite comme AC7** et laissée détachable.
- **D12 est la découverte de la spécification** : les deux contraintes d'unicité de `contacts` sont asymétriques à l'archivage, et une sonde IDE écrite sur le défaut `includeArchived: false` se tairait exactement dans le cas où l'utilisateur n'a **aucun recours** — le `409` permanent d'un porteur archivé, ni modifiable ni désarchivable. La note du 12 août ne pouvait pas le voir : elle raisonnait sur « un contact actif de la société ».
- Deux questions **nouvelles** sont ouvertes, dont Q1 bloque T1.

### File List

*(à remplir par `bmad-dev-story`)*
