# Epic 33 — registre des choix autonomes : fiches 33-4 à 33-10 (clé e334)

Tenu le 2026-11-06 par l'agent de fiche (Opus 5.5), en autonomie. Se verse **par union** au registre de l'epic. Chaque entrée : contexte, retenu, écartées, réversibilité.

## 33-4

- **C-e334-1 — Une date hors plage reçoit le refus d'une date mal formée.** *Contexte* : #606 attend « 400 comme `asOf` » ; or un corps `axum::Json` rend 422 en texte, un `Query` 400 en texte, une chaîne analysée 400 JSON. *Retenu* : borne posée à la désérialisation (`deserialize_with`), statut hérité du site ; critère : même refus que `2026-13-01`, jamais 500 ni 2xx, et marqueur `1000 à 9999`. *Écartées* : 400 JSON par une validation dans chaque handler (une trentaine de sites, oubliables, et un champ nouveau échapperait) ; un extracteur `Json` maison (changerait tous les rejets du dépôt). *Réversible* : oui, sans donnée.
- **C-e334-2 — Une fonction commune dans `kesh_core`, une constante recopiée dans `kesh-import`.** *Contexte* : `kesh-import` n'a aucune dépendance interne, invariant vérifié en CI. *Retenu* : `kesh_core::dates` ; constante locale à `kesh-import`, égalité testée côté `kesh-api`. *Écartée* : lever l'invariant. *Réversible* : oui.
- **C-e334-3 — Les analyseurs de fichiers entrent dans l'inventaire.** *Contexte* : le périmètre parlait de « dates reçues du client » ; un relevé en est une. *Retenu* : trois sites, trois lignes, l'erreur de date invalide existante. *Écartée* : les laisser (500 possible à l'import, ou année 0999 écrite). *Réversible* : oui. Recoupement léger avec 33-2, signalé.
- **C-e334-4 — Le test d'un corps se contente du champ date, et le marqueur le rend décidable.** *Contexte* : un corps partiel rend aussi 422 pour champ manquant, ce qui ferait un test muet. *Retenu* : `{ "champ": "+10000-01-01" }` et assertion sur le texte `1000 à 9999` ; la désérialisation s'arrête au champ invalide avant de constater un manquant. *Écartée* : monter un corps complet par site (coût, sans gain de preuve) ; le lot de réconciliation seul, dont l'extracteur a un message fixe, passe par une proposition valide. *Réversible* : oui.

## 33-5

- **C-e334-5 — `validate_required`, pays strict même inchangé.** *Contexte* : le contact garde un pays inconnu inchangé (`validate_optional_keeping`) parce que sa QR est facultative. *Retenu* : pour la société, créancière de toute QR, refuser : c'est la correction, et l'accepter annoncerait un succès avec une QR toujours en panne. *Écartée* : le patron du contact. *Réversible* : oui.
- **C-e334-6 — Pas de sélecteur commun.** *Contexte* : #910, liste ISO en quatre langues. *Retenu* : champ à deux lettres, comme contacts, et avis. *Écartée* : l'intégrer (déborde). #910 reste ouverte. *Réversible* : oui.
- **C-e334-7 — Route dédiée `PUT /companies/current/address`.** *Contexte* : patron existant d'une route par groupe de coordonnées, et `is_stub` réservé à l'assistant. *Retenu* : troisième route du patron, Admin. *Écartée* : une route générique ; réouvrir l'assistant. *Réversible* : oui. Compteurs du registre d'audit à recompter avec 33-7.

## 33-6

- **C-e334-8 — Trois sites clients traités ; les fournisseurs tracés.** *Contexte* : #450 croyait au dernier site ; il y en a trois dans `invoices.rs`, neuf côté fournisseur. *Retenu* : les trois clients, même patron ; issue demandée pour le fournisseur. *Écartée* : tout traiter (déborde, lecture à champs calculés, et 33-1 récrit le fichier). *Réversible* : oui.
- **C-e334-9 — La lecture vit dans l'enveloppe `cancel_settlement`, non dans `cancel_settlement_in_tx`.** *Contexte* : le dé-rapprochement réemploie la forme `_in_tx` et n'a pas besoin de la facture. *Retenu* : pas de lecture imposée à ce chemin. *Réversible* : oui.
- **C-e334-10 — Pas de crochet de test en production pour simuler le 404.** *Contexte* : la fenêtre tient entre deux requêtes, et le rouge n'est pas reproductible autrement. *Retenu* : rouge à la compilation, garde lexicale avec sonde, égalité de la réponse au `GET`. *Écartée* : injection de panne dans le handler. *Réversible* : oui. Écrit à la fiche : ce test n'établit pas la course, il établit qu'elle n'a plus de site.

## 33-7

- **C-e334-11 — Connexion, déconnexion, renouvellement ordinaire : non tracés.** *Contexte* : #435 pose le volume comme vrai sujet, et Guy n'a pas tranché. *Retenu* : les deux événements rares et lourds seulement ; fermeture de #435 avec ce choix écrit, et exemption maintenue au manuel. *Écartée* : tout tracer (noie les gestes comptables) ; garder #435 ouverte pour ce seul point. *Réversible* : oui. **Signalé au rapport.**
- **C-e334-12 — L'entrée de vol de jeton est au mieux.** *Contexte* : le 401 et la révocation priment. *Retenu* : `warn!` en cas d'échec. *Écartée* : refuser autrement. *Réversible* : oui.
- **C-e334-13 — `channel` et `mcp_key_id` pour le seul canal MCP.** *Contexte* : `services_motif.rs` compare le JSON exact sur une vingtaine de lignes ; et une ligne sans `channel` existe déjà par milliers. *Retenu* : absent pour une session. *Écartée* : `channel: "screen"` partout, qui laisserait croire qu'une ligne ancienne est un autre canal. *Réversible* : oui.
- **C-e334-14 — La clé d'API passe par `actor_type` (limite L2 levée), non par `channel`.** *Contexte* : `for_actor` existe, l'écran marque déjà « clé API ». *Retenu* : courte, intégrée (règle de l'epic). *Écartée* : `channel: "api_key"`, qui dirait deux fois. *Réversible* : oui.
- **C-e334-15 — Le type `Canal` ne change pas.** *Contexte* : sa `PartialEq` est testée à la clé près, et un `cle_id` y déplacerait des assertions sans rapport. *Retenu* : champ privé de `ContexteAcces`, accesseur `origine_trace`. *Réversible* : oui.

## 33-8

- **C-e334-16 — Avertir et faire confirmer, non refuser.** *Contexte* : #486 pose l'alternative ; l'archivage est ouvert au Comptable, le réglage à l'Administrateur ; un non-assujetti range ses comptes TVA désignés d'office ; #280 : « l'avertissement informe, il n'interdit pas ». *Retenu* : 409 `ACCOUNT_DESIGNATED_IN_SETTINGS`, puis `confirmDesignated`, patron `confirm_retype`. *Écartée* : refus sec. *Réversible* : oui. **Signalé au rapport.**
- **C-e334-17 — `accounts::archive` reste, `archive_guarded` s'ajoute.** *Contexte* : 14 appels, surtout des montages. *Retenu* : pas de diffusion. *Réversible* : oui.
- **C-e334-18 — La seconde face se corrige ici, pour l'inactif seulement.** *Contexte* : commentaire de #486 ; le type reconduit reste vérifié, #537 restant à part. *Retenu* : `changed` commande inactif et imputable. *Écartée* : ne plus rien valider à l'identique (agrandirait #537). *Réversible* : oui.

## 33-9

- **C-e334-19 — Aucune cause choisie à la fiche.** *Contexte* : la doc dit « route d'atterrissage », #287 dit « course », et le relevé de la 0.17.4 dit « vert rejoué seul ». *Retenu* : un protocole à trois verdicts, plus « non reproduit », et le verdict (b) reclasse la story. *Écartée* : poser le `poll` d'emblée : c'est la conduite que #287 elle-même refuse (« Établir laquelle avant de toucher au test »). *Réversible* : oui.
- **C-e334-20 — Le test gagne le sens inverse.** *Contexte* : « sans l'affaiblir », et le poll a besoin d'une contrepartie. *Retenu* : `'false'`, reload, fermé ; M1 à M4. *Réversible* : oui.

## 33-10

- **C-e334-21 — Un script séparé, `test-fast.sh` intact.** *Contexte* : consigne, et B1 acceptait « ou équivalent ». *Retenu* : `scripts/gate-ci.sh`. *Écartée* : une option de `test-fast.sh`. *Réversible* : oui.
- **C-e334-22 — `docker run --rm` en tâche de fond, sortie vers un fichier.** *Contexte* : les scripts de release utilisaient `docker start` et un conteneur nommé conservé ; et les consignes bannissent les `rm`. *Retenu* : code de retour et journal par le processus, aucun `docker rm`, aucun reste. *Réversible* : oui.
- **C-e334-23 — Les vérifications propres à une release restent hors dépôt, par `--apres`.** *Contexte* : elles changent à chaque release. *Retenu* : un crochet sourcé, et `--diff-attendu` pour les écarts admis. *Écartée* : tout dans le script (il deviendrait la somme des releases passées). *Réversible* : oui.
- **C-e334-24 — Les refus de sécurité passent avant toute action ; les verrous restent à l'appelant.** *Contexte* : Kesh tient des livres réels ; `~/kesh`, le NAS sont interdits. *Retenu* : motif de base, image exacte, chemins interdits, hôte fixe ; `flock` en amont comme les autres gates. *Écartée* : `verrous()` dans le script (propre à la station). *Réversible* : oui.
- **C-e334-25 — B2 n'est pas dans la story ; pas de ligne CHANGELOG utilisateur.** *Contexte* : le fichier d'epic porte B2 sur 33-10, la rétrospective, non, et `prompts/e33-commun.md` la tient. *Retenu* : la dire, sans code. *Réversible* : oui. **Signalé au rapport**, car le fichier d'epic ne se modifie pas ici.
