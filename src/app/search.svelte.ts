class SearchState {
  query = $state('');

  get active(): boolean {
    return this.query.trim().length > 0;
  }

  clear(): void {
    this.query = '';
  }
}

export const search = new SearchState();
