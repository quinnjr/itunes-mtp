import { CommonModule } from '@angular/common';
import { Component, inject } from '@angular/core';
import { FormsModule } from '@angular/forms';
import { FileUpload } from 'primeng/fileupload';
import { ItunesLibraryService } from '../core/services/itunes-library.service';
import { AppStateService } from '../core/services/app-state.service';
import { Playlist } from '../shared/models/library.model';

@Component({
  selector: 'app-home',
  imports: [
    CommonModule,
    FileUpload,
    FormsModule
  ],
  templateUrl: './home.component.html',
  styleUrl: './home.component.scss'
})
export class HomeComponent {
  // Inject services using inject() function for better zoneless compatibility
  private readonly itunesLibraryService = inject(ItunesLibraryService);
  private readonly appStateService = inject(AppStateService);

  // Public readonly signals from services
  public readonly libraryState = this.itunesLibraryService.libraryState;
  public readonly playlists = this.itunesLibraryService.getPlaylists.bind(this.itunesLibraryService);
  public readonly selectedPlaylists = this.itunesLibraryService.selectedPlaylists;
  public readonly appState = this.appStateService.appState;

  public onFileSelect(event: any): void {
    const file = event.files[0];
    if (!file) return;

    // Read the file
    const reader = new FileReader();
    reader.onload = (e) => {
      try {
        const xmlContent = e.target?.result as string;
        this.parseLibrary(xmlContent, file.name);
      } catch (error) {
        console.error('Failed to parse library:', error);
      }
    };
    reader.readAsText(file);
  }

  private async parseLibrary(xmlContent: string, fileName: string): Promise<void> {
    await this.itunesLibraryService.parseLibrary(xmlContent, fileName);
  }

  public clearLibrary(): void {
    this.itunesLibraryService.clearLibrary();
  }

  public togglePlaylist(playlist: Playlist): void {
    this.appStateService.togglePlaylistSelection(playlist);
  }

  public isPlaylistSelected(playlist: Playlist): boolean {
    return this.appStateService.isPlaylistSelected(playlist);
  }

  public selectAllPlaylists(): void {
    this.appStateService.selectAllPlaylists();
  }

  public deselectAllPlaylists(): void {
    this.appStateService.deselectAllPlaylists();
  }

  public async startSync(): Promise<void> {
    await this.appStateService.startSync();
  }

  // Computed properties for template
  public libraryLoaded(): boolean {
    return this.libraryState().isLoaded;
  }

  public libraryFileName(): string {
    return this.libraryState().fileName;
  }

  public trackCount(): number {
    return this.libraryState().trackCount;
  }

  public playlistCount(): number {
    return this.libraryState().playlistCount;
  }

  public isSyncing(): boolean {
    return this.appState().sync.isActive;
  }

  public syncProgress(): number {
    return this.appState().sync.percentage;
  }

  public canSync(): boolean {
    return this.appState().canSync;
  }
}
