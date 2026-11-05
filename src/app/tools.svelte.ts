/** The tool open over the overlay, with what it was opened with (a search result's text, for example). */
class Tools {
  current = $state<{ id: string; input?: string } | null>(null);

  open(id: string, input?: string): void {
    this.current = { id, input };
  }

  close(): void {
    this.current = null;
  }
}

export const tools = new Tools();
