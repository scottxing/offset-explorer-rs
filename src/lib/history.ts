// Browse history management
export interface HistoryItem {
  id: string;
  type: 'server' | 'topic';
  serverId: number | null;
  serverName?: string;
  topicName?: string;
  accessedAt: number;
}

const STORAGE_KEY = 'offset-explorer-history';
const MAX_HISTORY_ITEMS = 50;

/**
 * Get all history from localStorage
 */
export function getHistory(): HistoryItem[] {
  try {
    const stored = localStorage.getItem(STORAGE_KEY);
    if (!stored) return [];
    return JSON.parse(stored);
  } catch (error) {
    console.error('Failed to load history:', error);
    return [];
  }
}

/**
 * Save history to localStorage
 */
function saveHistory(history: HistoryItem[]): void {
  try {
    localStorage.setItem(STORAGE_KEY, JSON.stringify(history));
  } catch (error) {
    console.error('Failed to save history:', error);
  }
}

/**
 * Add a server access to history
 */
export function addHistoryServer(serverId: number, serverName: string): void {
  const history = getHistory();

  // Remove existing entries for the same server
  const filtered = history.filter(h => !(h.type === 'server' && h.serverId === serverId));

  const item: HistoryItem = {
    id: `server-${serverId}-${Date.now()}`,
    type: 'server',
    serverId,
    serverName,
    accessedAt: Date.now()
  };

  // Add new entry at the beginning
  filtered.unshift(item);

  // Keep only the most recent entries
  const trimmed = filtered.slice(0, MAX_HISTORY_ITEMS);
  saveHistory(trimmed);
}

/**
 * Add a topic access to history
 */
export function addHistoryTopic(serverId: number, serverName: string, topicName: string): void {
  const history = getHistory();

  // Remove existing entries for the same topic
  const filtered = history.filter(h => !(h.type === 'topic' && h.serverId === serverId && h.topicName === topicName));

  const item: HistoryItem = {
    id: `topic-${serverId}-${topicName}-${Date.now()}`,
    type: 'topic',
    serverId,
    serverName,
    topicName,
    accessedAt: Date.now()
  };

  // Add new entry at the beginning
  filtered.unshift(item);

  // Keep only the most recent entries
  const trimmed = filtered.slice(0, MAX_HISTORY_ITEMS);
  saveHistory(trimmed);
}

/**
 * Clear all history
 */
export function clearHistory(): void {
  saveHistory([]);
}

/**
 * Get history grouped by date (today, yesterday, this week, older)
 */
export function getHistoryGrouped(): { [key: string]: HistoryItem[] } {
  const history = getHistory();
  const grouped: { [key: string]: HistoryItem[] } = {
    today: [],
    yesterday: [],
    thisWeek: [],
    older: []
  };

  const now = Date.now();
  const oneDayMs = 24 * 60 * 60 * 1000;
  const todayStart = new Date();
  todayStart.setHours(0, 0, 0, 0);
  const yesterdayStart = todayStart.getTime() - oneDayMs;
  const weekStart = todayStart.getTime() - (7 * oneDayMs);

  for (const item of history) {
    if (item.accessedAt > todayStart.getTime()) {
      grouped.today.push(item);
    } else if (item.accessedAt > yesterdayStart) {
      grouped.yesterday.push(item);
    } else if (item.accessedAt > weekStart) {
      grouped.thisWeek.push(item);
    } else {
      grouped.older.push(item);
    }
  }

  return grouped;
}

/**
 * Format history item timestamp
 */
export function formatHistoryTime(timestamp: number): string {
  const date = new Date(timestamp);
  const now = new Date();
  const isToday = date.toDateString() === now.toDateString();

  if (isToday) {
    return date.toLocaleTimeString([], { hour: '2-digit', minute: '2-digit' });
  } else {
    return date.toLocaleDateString([], { month: 'short', day: 'numeric' });
  }
}
