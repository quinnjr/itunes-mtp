import { Component, inject } from '@angular/core';
import { CommonModule } from '@angular/common';
import { RouterOutlet } from '@angular/router';
import { MtpDeviceService } from './core/services/mtp-device.service';
import { AppStateService } from './core/services/app-state.service';
import { DeviceInfo, FileInfo } from './shared/models/device.model';

@Component({
  selector: 'app-root',
  standalone: true,
  imports: [CommonModule, RouterOutlet],
  templateUrl: './app.component.html',
  styleUrl: './app.component.scss'
})
export class AppComponent {
  // Inject services using inject() function for better zoneless compatibility
  private readonly mtpDeviceService = inject(MtpDeviceService);
  private readonly appStateService = inject(AppStateService);

  // Public readonly signals from services
  public readonly devices = this.mtpDeviceService.devices;
  public readonly connectionState = this.mtpDeviceService.connectionState;
  public readonly deviceFiles = this.mtpDeviceService.deviceFiles;
  public readonly currentFolder = this.mtpDeviceService.currentFolder;
  public readonly appState = this.appStateService.appState;

  // ngOnInit not needed - services initialized in constructors

  public async refreshDevices(): Promise<void> {
    await this.mtpDeviceService.refreshDevices();
  }

  public async selectDevice(device: DeviceInfo): Promise<void> {
    await this.mtpDeviceService.connectToDevice(device);
  }

  public async browseFolder(file: FileInfo): Promise<void> {
    await this.mtpDeviceService.browseFolder(file);
  }

  public async goUpFolder(): Promise<void> {
    await this.mtpDeviceService.goUpFolder();
  }

  public getSyncStatus(): string {
    return this.appStateService.getSyncStatusMessage();
  }

  public getSelectedDevice(): DeviceInfo | null {
    return this.appStateService.getConnectedDevice();
  }
}
