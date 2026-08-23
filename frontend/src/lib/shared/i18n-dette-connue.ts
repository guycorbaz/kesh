/**
 * Dette i18n connue — clés demandées par le frontend et absentes des QUATRE catalogues.
 *
 * ⚠️ **CETTE LISTE EST VIDE DEPUIS LA STORY 23-6 (2026-08-22), ET C'EST SON ÉTAT NORMAL.**
 * Elle a compté **317 clés** au kickoff de l'Epic 23 ; les stories 23-1b à 23-6 les ont
 * toutes résorbées. La garde `i18n-keys.test.ts` est donc désormais **inconditionnelle** :
 * toute clé demandée par le code et absente d'un catalogue fait rougir le gate, sans
 * dérogation possible.
 *
 * ⚠️ **N'AJOUTEZ PAS D'ENTRÉE ICI POUR FAIRE PASSER UN GATE.** C'est le seul usage que
 * cette liste ne doit jamais avoir. Une clé manquante se répare en écrivant sa valeur dans
 * les quatre `crates/kesh-i18n/locales/<locale>/messages.ftl` — c'est le travail que l'epic entier
 * a consisté à faire, et il tient en quatre lignes par clé.
 *
 * ⚠️ **Le défaut que tout ceci prévient est MUET, et c'est pourquoi la liste reste ici
 * plutôt que d'être supprimée.** `i18nMsg(clé, repli)` retombe silencieusement sur son
 * second argument — du français en dur — quand la clé manque, et `loader.rs` charge `fr-CH`
 * comme base de repli des trois autres locales. Un oubli de traduction ne produit donc ni
 * erreur, ni avertissement, ni clé brute à l'écran : il produit **du français correct,
 * servi à un germanophone, avec tous les gates au vert**. Un fichier vide portant cette
 * explication vaut mieux qu'un fichier absent dont plus personne ne connaît la raison.
 *
 * Historique : plan `_bmad-output/planning-artifacts/epic-23-dette-i18n.md`, issues
 * [#316] (KF-040, 285 clés absentes des quatre catalogues) et [#283] (57 clés absentes
 * de trois locales sur quatre).
 */
export const DETTE_CONNUE: readonly string[] = [];
