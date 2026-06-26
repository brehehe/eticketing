/**
 * Utility functions for client-side exporting of report data to Excel (CSV) and PDF (Printable view).
 */

export function exportToExcel(headers: string[], rows: any[][], filename: string) {
  let csv = '\ufeff'; // UTF-8 BOM so Excel opens with proper character encoding
  csv += 'sep=,\n';   // Specify comma as delimiter for Excel

  // Header row
  csv += headers.map(h => `"${String(h).replace(/"/g, '""')}"`).join(',') + '\n';

  // Data rows
  rows.forEach(row => {
    csv += row.map(cell => `"${String(cell ?? '').replace(/"/g, '""')}"`).join(',') + '\n';
  });

  const blob = new Blob([csv], { type: 'text/csv;charset=utf-8;' });
  const url = URL.createObjectURL(blob);
  const link = document.createElement('a');
  link.setAttribute('href', url);
  link.setAttribute('download', `${filename}.csv`);
  document.body.appendChild(link);
  link.click();
  document.body.removeChild(link);
}

export function exportToPDF(
  title: string,
  dateRange: string,
  stats: { label: string; value: string }[],
  headers: string[],
  rows: any[][]
) {
  const printWindow = window.open('', '_blank');
  if (!printWindow) {
    alert('Pop-up blockir terdeteksi. Silakan izinkan pop-up untuk mencetak PDF.');
    return;
  }

  let html = `
    <!DOCTYPE html>
    <html lang="id">
    <head>
      <title>${title}</title>
      <meta charset="utf-8">
      <style>
        @import url('https://fonts.googleapis.com/css2?family=Inter:wght@400;500;600;700&display=swap');
        body {
          font-family: 'Inter', -apple-system, sans-serif;
          color: #1f2937;
          margin: 40px;
          line-height: 1.5;
          font-size: 13px;
        }
        .header {
          margin-bottom: 24px;
          border-bottom: 2px solid #f3f4f6;
          padding-bottom: 16px;
        }
        .header h1 {
          font-size: 22px;
          font-weight: 700;
          margin: 0 0 6px 0;
          color: #111827;
          letter-spacing: -0.025em;
        }
        .header .date-range {
          font-size: 13px;
          color: #6b7280;
          font-weight: 500;
        }
        .stats-grid {
          display: grid;
          grid-template-columns: repeat(4, 1fr);
          gap: 16px;
          margin-bottom: 30px;
        }
        .stat-card {
          border: 1px solid #e5e7eb;
          border-radius: 8px;
          padding: 12px 16px;
          background: #f9fafb;
        }
        .stat-label {
          font-size: 10px;
          font-weight: 600;
          text-transform: uppercase;
          color: #6b7280;
          letter-spacing: 0.05em;
          display: block;
          margin-bottom: 4px;
        }
        .stat-value {
          font-size: 16px;
          font-weight: 700;
          color: #111827;
        }
        table {
          width: 100%;
          border-collapse: collapse;
          margin-bottom: 30px;
        }
        th {
          background-color: #f9fafb;
          color: #374151;
          font-weight: 600;
          text-align: left;
          padding: 10px 12px;
          border-bottom: 2px solid #e5e7eb;
        }
        td {
          padding: 10px 12px;
          border-bottom: 1px solid #f3f4f6;
          color: #4b5563;
        }
        tr:nth-child(even) td {
          background-color: #fafafa;
        }
        .text-right {
          text-align: right;
        }
        .text-center {
          text-align: center;
        }
        @media print {
          body {
            margin: 20px;
          }
          .stats-grid {
            grid-template-columns: repeat(4, 1fr) !important;
          }
        }
      </style>
    </head>
    <body>
      <div class="header">
        <h1>${title}</h1>
        <div class="date-range">Periode: ${dateRange}</div>
      </div>
  `;

  if (stats && stats.length > 0) {
    html += '<div class="stats-grid">';
    stats.forEach(s => {
      html += `
        <div class="stat-card">
          <span class="stat-label">${s.label}</span>
          <span class="stat-value">${s.value}</span>
        </div>
      `;
    });
    html += '</div>';
  }

  html += `
    <table>
      <thead>
        <tr>
  `;

  headers.forEach(h => {
    html += `<th>${h}</th>`;
  });

  html += `
        </tr>
      </thead>
      <tbody>
  `;

  rows.forEach(row => {
    html += '<tr>';
    row.forEach(cell => {
      let cellClass = '';
      const cellStr = String(cell ?? '');
      // Format number and currency right alignment
      if (cellStr.startsWith('Rp') || /^-?Rp/.test(cellStr) || /^[0-9,.-]+%?$/.test(cellStr) || /^\d+$/.test(cellStr)) {
        cellClass = ' class="text-right"';
      }
      html += `<td${cellClass}>${cellStr}</td>`;
    });
    html += '</tr>';
  });

  html += `
      </tbody>
    </table>
    <script>
      window.onload = function() {
        window.print();
        setTimeout(function() { window.close(); }, 500);
      };
    </script>
    </body>
    </html>
  `;

  printWindow.document.write(html);
  printWindow.document.close();
}
