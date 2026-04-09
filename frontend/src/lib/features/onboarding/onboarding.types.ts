/** État d'onboarding retourné par l'API. */
export interface OnboardingState {
	stepCompleted: number;
	isDemo: boolean;
	uiMode: 'guided' | 'expert' | null;
}
