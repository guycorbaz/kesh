/** État d'onboarding retourné par l'API. */
export interface OnboardingState {
	stepCompleted: number;
	isDemo: boolean;
	uiMode: 'guided' | 'expert' | null;
	/** `true` tant que la company courante est un placeholder (bootstrap DB vide
	 * ou wizard avant complétion). Sert au nudge de renommage non-bloquant. */
	isStub: boolean;
}
