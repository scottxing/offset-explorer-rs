// Favorite items management
export interface FavoriteItem {
  id: string;
  type: 'server' | 'topic';
  serverId: number | null;
  serverName?: string;
  topicName?: string;
  createdAt: number;
}

const STORAGE_KEY = 'offset-explorer-favorites';

/**
 * Get all favorites from localStorage
 */
export function getFavorites(): FavoriteItem[] {
  try {
    const stored = localStorage.getItem(STORAGE_KEY);
    if (!stored) return [];
    return JSON.parse(stored);
  } catch (error) {
    console.error('Failed to load favorites:', error);
    return [];
  }
}

/**
 * Save favorites to localStorage
 */
function saveFavorites(favorites: FavoriteItem[]): void {
  try {
    localStorage.setItem(STORAGE_KEY, JSON.stringify(favorites));
  } catch (error) {
    console.error('Failed to save favorites:', error);
  }
}

/**
 * Add a server to favorites
 */
export function addFavoriteServer(serverId: number, serverName: string): FavoriteItem {
  const favorites = getFavorites();
  const favorite: FavoriteItem = {
    id: `server-${serverId}-${Date.now()}`,
    type: 'server',
    serverId,
    serverName,
    createdAt: Date.now()
  };

  favorites.push(favorite);
  saveFavorites(favorites);
  return favorite;
}

/**
 * Add a topic to favorites
 */
export function addFavoriteTopic(serverId: number, serverName: string, topicName: string): FavoriteItem {
  const favorites = getFavorites();
  const favorite: FavoriteItem = {
    id: `topic-${serverId}-${topicName}-${Date.now()}`,
    type: 'topic',
    serverId,
    serverName,
    topicName,
    createdAt: Date.now()
  };

  favorites.push(favorite);
  saveFavorites(favorites);
  return favorite;
}

/**
 * Remove a favorite by ID
 */
export function removeFavorite(id: string): void {
  const favorites = getFavorites();
  const filtered = favorites.filter(f => f.id !== id);
  saveFavorites(filtered);
}

/**
 * Check if an item is favorited
 */
export function isFavorited(type: 'server' | 'topic', serverId: number, topicName?: string): boolean {
  const favorites = getFavorites();

  return favorites.some(f => {
    if (f.type !== type) return false;
    if (f.serverId !== serverId) return false;
    if (type === 'topic' && f.topicName !== topicName) return false;
    return true;
  });
}

/**
 * Get favorite ID for an item
 */
export function getFavoriteId(type: 'server' | 'topic', serverId: number, topicName?: string): string | null {
  const favorites = getFavorites();

  const favorite = favorites.find(f => {
    if (f.type !== type) return false;
    if (f.serverId !== serverId) return false;
    if (type === 'topic' && f.topicName !== topicName) return false;
    return true;
  });

  return favorite?.id || null;
}

/**
 * Clear all favorites
 */
export function clearFavorites(): void {
  saveFavorites([]);
}
