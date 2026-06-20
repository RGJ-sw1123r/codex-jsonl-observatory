export interface ExportWorklogResponse {
  status: 'exported'
  bundle_path: string
  generated_files: string[]
  refreshed: boolean
  folder_opened: boolean
  folder_open_error: string | null
}
