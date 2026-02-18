const _isMacOS = navigator.platform.toUpperCase().includes('MAC') ||
  navigator.userAgent.toUpperCase().includes('MAC');

export function isMacOS(): boolean {
  return _isMacOS;
}

export function getModifierKey(): string {
  return _isMacOS ? 'Cmd' : 'Ctrl';
}
