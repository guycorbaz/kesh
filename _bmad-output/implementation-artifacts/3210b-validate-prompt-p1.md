# Prompt — validation P1, Story 32-10b (l'écran des prompts du serveur MCP)

*Versionné le 2026-10-27. Passe **complète**, première sur huit. **Deux lentilles, R et F**, chacune en contexte frais,
**Sonnet**. Lecture seule stricte.*

Dépôt `/home/gcorbaz/devel/kesh-32-10b`, branche `story/32-10b` (fiche 32-10a + `origin/main` `ed981c7d` fusionné,
merge `42680725` : la 32-8 complète, 32-8c comprise ; la **32-10a est validée, non développée** ; la 32-7 n'est pas
fusionnée). Fiche : `_bmad-output/implementation-artifacts/32-10b-ecran-prompts.md` ; registre :
`_bmad-output/implementation-artifacts/epic-32-choix-autonomes-3210b.md` (C-32-10b-1 à 16). Contexte :
`_bmad-output/planning-artifacts/epic-32-serveur-mcp.md` (§ *Décisions du Project Lead*, § *32-10*, *Réponses de Guy*) ;
**la fiche 32-10a**, `_bmad-output/implementation-artifacts/32-10a-prompts-en-base.md` (AC-i6, AC-i7, AC-i8, AC-ii2,
AC-ii5 à AC-ii7, AC-iii6 à AC-iii8, AC-iii11, inventaire P1 à P27, § *La frontière avec la 32-10b*, § *Le manuel*,
*Dev Notes*) et son registre `epic-32-choix-autonomes-3210a.md` ; **le modèle voisin, fusionné** : la fiche
`32-8-validation-avant-envoi.md` partie C (AC-C1 à AC-C11), le registre `epic-32-choix-autonomes-328.md` (C-32-8-5,
C-32-8-20, C-32-8-21, C-32-8-71) et le code `frontend/src/lib/features/mcp-sends/`, `frontend/src/routes/(app)/admin/
mcp/sends/`, `frontend/tests/e2e/mcp-sends.spec.ts` ; **la fiche 32-7, en lecture seule dans un autre worktree** :
`/home/gcorbaz/devel/kesh-32-7/_bmad-output/implementation-artifacts/32-7-ecran-serveur-mcp.md` (AC-2, AC-14, AC-15,
AC-18, AC-20). Règles : `CLAUDE.md` (§ *Review Iteration Rule*, *Inventorier les sites NON RÉSOLUS*, *Recette
visuelle*, *Le prompt d'une passe doit NOMMER le manuel*, *Recompter ses propres comptes rendus*, *Propagation
post-patch*, *Règle de splitting préventif*), `docs/testing.md`, `docs/agents-orchestration.md`.

**Contraintes de la story, à tenir pour vraies et à contrôler** : au plus trois modules (frontend, `kesh-i18n`, manuel
d'administration) ; la 32-10b n'ajoute **aucune route** ni code Rust (fiche 32-10a, § *La frontière*) ; un prompt est
une instruction donnée à un agent qui lit des livres réels : l'écran doit montrer **exactement** le texte servi,
caractères cachés rendus visibles, et l'approbation se faire sur la version **vue**, avec contrôle de version ; la
marque d'un prompt d'agent visible à l'écran comme dans le texte servi ; le retour au standard sans écrasement ;
l'Administrateur seul voit et décide ; captures en thème clair seulement ; Hermes hors périmètre ; aucune donnée réelle.
Ce que la fiche **prescrit** n'existe pas encore : son absence n'est pas un défaut. Ce que la fiche cite **de la fiche
32-10a** est prévu, non codé : juge sa cohérence avec cette fiche, non sa présence dans le code.

⛔ **Interdits** : n'écris **aucun** fichier dans le dépôt ni dans `/home/gcorbaz/devel/kesh-32-7` ; n'exécute **ni**
`cargo`, `npm`, `npx`, `node`, `make`, `docker`, `curl`, **ni aucun interpréteur** (`python`, `perl`, `awk`, `sh -c`…),
**ni redirection** (`>`, `>>`, `tee`) hors de l'écriture de ton rapport par l'outil d'écriture, **ni**
`scripts/prepare-release.sh`, `scripts/test-fast.sh`, `scripts/regen-test-schema.sh` ou tout autre script ; **aucune**
commande `gh` ni `git` qui écrit (`commit`, `push`, `checkout`, `switch`, `reset`, `stash`, `merge`, `rebase`, `tag`,
`worktree`) ; n'ouvre jamais `~/kesh-test/`. Commandes permises : `grep`, `sed -n`, `ls`, `wc`, `cut`, `sort`, `head`,
`tail`, `git log`, `git show`, `git diff`, `pdftotext` vers la sortie standard. Le seul fichier que tu écris est ton
rapport.

## Les trois axes obligatoires, pour les deux lentilles

1. ⛔ **Chaque affirmation de présence ou d'absence, prouvée par une sortie `grep -nF` copiée** (ou `sed -n` de la
   ligne citée). Dresse d'abord la liste des motifs et des mesures : `grep -nE "grep -|→ \*\*|au relevé|mesure"
   _bmad-output/implementation-artifacts/32-10b-ecran-prompts.md`. Pour chaque mesure (faits 1 à 11, AC-6 — les
   **16** jetons des listes de `is_hidden_format` —, AC-13, AC-15, AC-19), exécute-la dans son périmètre et copie la
   sortie ; une mesure que tu ne reproduis pas, un témoin « existant » absent de la ligne citée, une ligne citée qui
   ne dit pas ce que la fiche lui prête, sont des findings.
2. ⛔ **Le manuel, au `.tex` ET au PDF aplati.** `grep -n 'sec:mcp-envois\|routes à verbe mutant\|section{Sécurité}\|
   section{Monitoring' docs/manual/fr/admin-manual.tex` ; `pdftotext docs/manual/fr/admin-manual.pdf - | tr '\n' ' ' |
   tr -s ' ' | grep -oiE '.{60}(envois du serveur MCP|routes à verbe mutant|prompt).{80}'`. L'AC-18 prescrit-elle tout
   ce que la fiche 32-10a laisse à la 32-10b (§ *Le manuel*, « Ce qu'elle laisse à la 32-10b », et P26) ? Chaque phrase
   prescrite décrit-elle ce que la fiche 32-10a dit du code — non une intention ? La phrase « 122 des 128 » est-elle
   déjà fausse (recompte au registre `crates/kesh-api/tests/audit_route_registry.rs`) ?
3. ⛔ **Recompter les décomptes de la fiche depuis sa source** : 20 AC (A 4, B 2, C 8, D 6), 9 tâches (T0 à T8), 16
   mutations (M1 à M16), E1 à E15, C-32-10b-1 à 16 (chacun cité par la fiche existe au registre et y dit la même chose),
   7 points pour Guy, 4 demandes ; chaque total contre sa ventilation (`CLAUDE.md` § *Recompter*).

## Lentille R — la spécification contre le code et contre la fiche 32-10a

1. **Les faits 1 à 11 et § *Ce qui est lu et cité*** : chaque numéro de ligne, ouvert.
2. **Le contrat avec la 32-10a** : chaque ligne du tableau « Ce que dit la fiche 32-10a » contre l'AC qu'il cite ;
   les règles d'approbation (AC-i7 : dernière version, états terminaux), de version en vigueur (AC-i6), de retour au
   standard (AC-iii11 (d), AC-i10), les codes d'erreur (AC-iii6, AC-iii8) ; la portée qui sert un prompt (C-32-10a-16) ;
   la forme de la marque (*Dev Notes — La marque*). Une règle de l'écran qui contredit ou refait la 32-10a est un
   finding. La demande **D1** est-elle réalisable dans les modules déjà comptés par la 32-10a-iii (le `routes` et le
   `mcp` de sa table) sans affaiblir une garde de la 32-6 ou de la 32-10a (`prompts_lus_par_leur_service`,
   `mcp_sans_pool`, la chaîne des types de l'AC-ii8) ? La variante B l'est-elle sans toucher au Rust ?
3. **Le modèle 32-8c** : la fiche reprend-elle juste ses motifs (fabriques de corps seules, version lue, 409 et
   *Recharger*, `ecran_sans_html_brut`, garde des replis, `FAMILLES_RESOLUES`, `sitesTotal`, E2E à contexte neuf) ?
   S'en écarte-t-elle sans le dire (C-32-10b-3 contre C-32-8-20 ; C-32-10b-9) ?
4. **Les caractères cachés** (AC-6) : le miroir de `is_hidden_in_text` et `is_hidden_in_line` (`crates/kesh-core/
   src/text.rs:86-133`) est-il exact — Cc selon le mode, Cf, Co, Cn, les deux listes, Zl/Zp ? Les classes `\p{gc=…}`
   de JavaScript couvrent-elles ce que la fiche dit ? La garde `listes_egales_a_kesh_core` est-elle réalisable sur la
   forme réelle du `matches!` (plages `..=`, commentaires) ? La liste `admis` reprend-elle l'angle mort écrit
   (`text.rs:79-85`) ?
5. **L'i18n** : préfixe `mcp-prompts-` contre le catalogue (`grep -n '^mcp-prompt' crates/kesh-i18n/locales/fr-CH/
   messages.ftl`) et contre ce que la 32-10a-iii écrira ; les variantes (a) et (b) de l'AC-15 contre `mcp-sends-i18n.
   test.ts` et la fiche 32-7 (AC-14) ; G3-bis (`crates/kesh-i18n/src/loader.rs`) — un texte prescrit écrit-il
   « Administration → … » dans un catalogue ?
6. **L'E2E** (AC-16) contre `mcp-sends.spec.ts` et la fiche 32-10a : la clé à deux portées se crée-t-elle par la route
   de la 32-5c (`crates/kesh-api/src/routes/` — lis la validation des portées) ? `proposer_prompt` et `prompts/get` ont-ils
   la forme supposée (AC-iii2, AC-iii3, AC-ii6 de la 32-10a) ? Un préréglage Comptable existe-t-il (`frontend/tests/
   e2e/helpers/`) ?

## Lentille F — la propriété centrale, la sécurité, la faisabilité

1. **La propriété : ce qui est approuvé est ce qui a été vu, et ce qui est servi est ce qui a été approuvé.** Cherche
   un chemin **hors** de l'inventaire E1 à E15 par lequel l'Administrateur approuve, enregistre ou rétablit un texte
   différent de celui que l'agent recevra : rendu, relecture, cache du navigateur, ordre des requêtes, double clic,
   deux onglets, un standard modifié par un amorçage entre la lecture et le clic, une restauration, une version
   d'administrateur venue d'un autre onglet. Chaque ligne de l'inventaire est-elle fermée par ce qu'elle dit ?
2. **La marque** : peut-elle manquer, être imitée par le texte d'un agent sans que l'écran le montre, être prise pour
   un ornement ? La règle « encadrée seulement si le texte commence par la marque » est-elle sûre ?
3. **Les caractères cachés** : un caractère qui échappe au signalement **et** au serveur (homoglyphes, `Zs`, `Mn`
   hors liste) ; la pastille `⟦U+XXXX⟧` peut-elle être imitée par un texte littéral (un agent écrit `⟦U+202E⟧`) et
   tromper l'Administrateur ? (La 32-8c l'a écrit pour ses `\u{…}`.) Le rendu `pre` + `dir="ltr"` neutralise-t-il un
   caractère bidirectionnel **visible** (lettres arabes, hébraïques) ?
4. **Les rôles** : un Comptable, une clé d'API, une clé MCP — que voit-on, que peut-on faire ? L'écran offre-t-il un
   geste que la route refuserait, ou l'inverse ?
5. **Découpage et faisabilité** : recompte les modules selon la convention écrite (trois) ; chaque fichier de
   l'esquisse et des AC est-il dans un module compté ? La variante (b) de l'AC-15, qui touche la garde des envois,
   reste-t-elle dans le frontend ? Une tâche suppose-t-elle un fichier, une fonction, un composant (`Dialog`,
   `Button`, pastille) qui n'existe pas ?
6. **Tests et mutations** : chaque AC vérifiable tel qu'écrit ; chaque mutation M1 à M16 réalisable et tuée par le test
   **nommé** — une passerait-elle à vide (un test qui ne monte pas le cas) ? L'E2E (a) prouve-t-il vraiment « valeur
   vue = valeur servie » (le `textContent` d'un `pre` à pastilles ; un texte sans pastille constaté) ?
7. **Le registre et les questions** : chaque choix dit-il la même chose que la fiche ? Une question pour Guy est-elle
   déjà tranchée (décisions de l'epic, de la 32-10a) ? Une décision de Guy (§ *Réponses de Guy*) est-elle contredite ?

## Rapport

Écris ton rapport — et rien d'autre — dans `/home/gcorbaz/devel/kesh-gate-logs/3210b-validate-p1-R.md` (lentille R) ou
`/home/gcorbaz/devel/kesh-gate-logs/3210b-validate-p1-F.md` (lentille F). Contenu :

1. **Axes exercés** et **axes non exercés** (avec la raison) — liste explicite, axe par axe, y compris les trois axes
   obligatoires. Un « 0 finding » sans cette liste ne compte pas comme passe.
2. **Findings** numérotés (`R1`, `R2`… ou `F1`, `F2`…), chacun avec : sévérité (CRITICAL, HIGH, MEDIUM, LOW), le site
   de la fiche (section, AC), la **preuve** (commande exécutée et sortie copiée — `grep -nF` pour toute présence ou
   absence —, ou ligne lue), et la correction proposée.
3. Le décompte `CRITICAL/HIGH/MEDIUM/LOW`, recompté depuis ta liste.
