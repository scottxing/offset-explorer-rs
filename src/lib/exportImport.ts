// Data export and import utilities
export interface ExportOptions {
  format: 'json' | 'csv';
    includeKey: boolean;
    includeValue: boolean;
    includeHeaders: boolean;
    includeTimestamp: boolean;
  }

export interface ImportResult {
  success: boolean;
  messagesExported?: number;
  errorMessage?: string;
}

/**
 * Convert messages to CSV format
 */
function messagesToCSV(messages: any[], options: ExportOptions): string {
  const headers = options.includeHeaders ? 'Key,Partition,Offset,Timestamp,Value' : 'Partition,Offset,Timestamp';
  const csvRows = [headers];

  // Add message rows
  for (const msg of messages) {
    const row: string[] = [];

    if (options.includeKey && msg.key) {
      row.push(`"${escapeCSV(msg.key)}"`);
    } else {
      row.push(''); // Empty key
    }

    row.push(String(msg.partition));
    row.push(String(msg.offset));
    row.push(options.includeTimestamp ? new Date(msg.timestamp).toISOString() : '');
    row.push(`"${escapeCSV(msg.value)}"`);

    csvRows.push(row.join(','));
  }

  return csvRows.join('\n');
}

/**
 * Escape CSV field value
 */
function escapeCSV(value: string): string {
  if (!value) return '';
  const withQuotes = value.includes(',') || value.includes('"') || value.includes('\n') || value.includes('\r');
  if (withQuotes) {
    return `"${value.replace(/"/g, '""')}"`;
  }
  return value;
}

/**
 * Convert messages to JSON format
 */
function messagesToJSON(messages: any[], options: ExportOptions): string {
  return JSON.stringify(messages, null, 2);
}

/**
 * Export messages to file
 */
export async function exportMessages(messages: any[], options: ExportOptions): Promise<void> {
  let data: string;
  let filename: string;
  let mimeType: string;

  if (options.format === 'csv') {
    data = messagesToCSV(messages, options);
    filename = 'messages-export.csv';
    mimeType = 'text/csv';
  } else {
    data = messagesToJSON(messages, options);
    filename = 'messages-export.json';
    mimeType = 'application/json';
  }

  // Create download link
  const blob = new Blob([data], { type: mimeType });
  const url = URL.createObjectURL(blob);
  const a = document.createElement('a');
  a.href = url;
  a.download = filename;
  document.body.appendChild(a);
  a.click();
  document.body.removeChild(a);
  URL.revokeObjectURL(url);
}

/**
 * Parse CSV data for import
 */
function parseCSV(csvText: string): any[] {
  const lines = csvText.split('\n');
  if (lines.length === 0) return [];

  // Detect headers
  const firstLine = lines[0];
  const hasHeaders = firstLine.toLowerCase().includes('key');

  // Parse headers or use defaults
  const headers = hasHeaders
    ? firstLine.split(',').map(h => h.trim())
    : ['Key', 'Partition', 'Offset', 'Timestamp', 'Value'];

  const result = [];

  for (let i = hasHeaders ? 1 : 0; i < lines.length; i++) {
    const line = lines[i];
    const values: string[] = [];

    // Simple CSV parser (doesn't handle quoted fields with commas inside)
    let inQuotes = false;
    let currentValue = '';

    for (let char of line) {
      if (char === '"') {
        inQuotes = !inQuotes;
      } else if (char === ',' && !inQuotes && currentValue) {
        values.push(currentValue.trim());
        currentValue = '';
      } else if (char === ',' && !inQuotes) {
        values.push(currentValue.trim());
        currentValue = '';
      } else {
        currentValue += char;
      }
    }

    if (currentValue) {
      values.push(currentValue.trim());
    }

    // Ensure we have the right number of fields
    while (values.length < headers.length) {
      values.push(''); // Empty field
    }

    if (values.length === headers.length) {
      result.push(values.reduce((acc: Record<string, string>, val, idx) => {
        acc[headers[idx]] = val;
        return acc;
      }, {}));
    }
  }

  return result;
}

/**
 * Import messages from file
 */
export async function importMessages(file: File): Promise<ImportResult> {
  const text = await file.text();

  try {
    const extension = file.name.split('.').pop()?.toLowerCase();

    if (extension === 'csv') {
      const messages = parseCSV(text);
      return { success: true, messagesExported: messages.length };
    } else if (extension === 'json') {
      const messages = JSON.parse(text);
      return { success: true, messagesExported: messages.length };
    } else {
      return { success: false, errorMessage: 'Unsupported file format. Please use JSON or CSV.' };
    }
  } catch (error) {
    return { success: false, errorMessage: error instanceof Error ? error.message : 'Unknown error' };
  }
}

/**
 * Trigger file download dialog
 */
export function triggerFileExport(options: ExportOptions): void {
  // Create file input element
  const input = document.createElement('input');
  input.type = 'file';
  input.accept = options.format === 'csv' ? '.csv' : '.json';

  // Trigger file selection dialog
  input.click();

  // Listen for file selection
  input.onchange = async (e) => {
    const file = (e.target as HTMLInputElement).files?.[0];
    if (!file) return;

    const result = await importMessages(file);

    if (result.success) {
      // Import would be handled by the backend
      console.log('Imported', result.messagesExported, 'messages');
    } else if (result.errorMessage) {
      alert('Import failed: ' + result.errorMessage);
    }

    // Clean up
    input.remove();
  };

  // Trigger click to start file upload
  input.click();
}
