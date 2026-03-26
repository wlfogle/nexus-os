import { describe, it, expect, vi, beforeEach } from 'vitest';
import { render, screen, fireEvent, waitFor } from '@testing-library/react';
import { VirtualMachine, VmState } from '@/types';
import VmCard from '../VmCard';

// Mock data
const mockVm: VirtualMachine = {
  id: 'vm-1',
  name: 'Test VM',
  state: 'Running' as VmState,
  memory: 2048, // 2GB in MB
  vcpus: 2,
  disk_size: 50, // 50GB
  os_type: 'Linux',
  os_variant: 'Ubuntu 22.04',
  created_at: '2024-01-01T10:00:00Z',
  last_started: '2024-01-02T09:00:00Z',
  snapshots: [],
  network_interfaces: [],
  storage_devices: [],
};

const mockHandlers = {
  onStart: vi.fn(),
  onStop: vi.fn(),
  onDelete: vi.fn(),
};

describe('VmCard', () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });

  it('renders VM information correctly', () => {
    render(<VmCard vm={mockVm} {...mockHandlers} />);

    expect(screen.getByText('Test VM')).toBeInTheDocument();
    expect(screen.getByText('Running')).toBeInTheDocument();
    expect(screen.getByText('2 vCPUs')).toBeInTheDocument();
    expect(screen.getByText('2GB RAM')).toBeInTheDocument();
    expect(screen.getByText('Linux (Ubuntu 22.04)')).toBeInTheDocument();
  });

  it('shows correct state styling for running VM', () => {
    render(<VmCard vm={mockVm} {...mockHandlers} />);
    
    const statusBadge = screen.getByText('Running');
    expect(statusBadge).toHaveClass('bg-green-100');
  });

  it('shows correct state styling for stopped VM', () => {
    const stoppedVm = { ...mockVm, state: 'Stopped' as VmState };
    render(<VmCard vm={stoppedVm} {...mockHandlers} />);
    
    const statusBadge = screen.getByText('Stopped');
    expect(statusBadge).toHaveClass('bg-gray-100');
  });

  it('shows Stop button for running VM', () => {
    render(<VmCard vm={mockVm} {...mockHandlers} />);
    
    const stopButton = screen.getByText('Stop');
    expect(stopButton).toBeInTheDocument();
    expect(stopButton).toBeEnabled();
  });

  it('shows Start button for stopped VM', () => {
    const stoppedVm = { ...mockVm, state: 'Stopped' as VmState };
    render(<VmCard vm={stoppedVm} {...mockHandlers} />);
    
    const startButton = screen.getByText('Start');
    expect(startButton).toBeInTheDocument();
    expect(startButton).toBeEnabled();
  });

  it('calls onStop when Stop button is clicked', async () => {
    render(<VmCard vm={mockVm} {...mockHandlers} />);
    
    const stopButton = screen.getByText('Stop');
    fireEvent.click(stopButton);

    await waitFor(() => {
      expect(mockHandlers.onStop).toHaveBeenCalledTimes(1);
    });
  });

  it('calls onStart when Start button is clicked', async () => {
    const stoppedVm = { ...mockVm, state: 'Stopped' as VmState };
    render(<VmCard vm={stoppedVm} {...mockHandlers} />);
    
    const startButton = screen.getByText('Start');
    fireEvent.click(startButton);

    await waitFor(() => {
      expect(mockHandlers.onStart).toHaveBeenCalledTimes(1);
    });
  });

  it('calls onDelete when Delete option is clicked', async () => {
    render(<VmCard vm={mockVm} {...mockHandlers} />);
    
    // Click on menu button
    const menuButton = screen.getByRole('button', { name: '' }); // Menu button with no text
    fireEvent.click(menuButton);
    
    // Click on Delete option
    const deleteButton = screen.getByText('Delete VM');
    fireEvent.click(deleteButton);

    await waitFor(() => {
      expect(mockHandlers.onDelete).toHaveBeenCalledTimes(1);
    });
  });

  it('disables buttons during loading state', async () => {
    render(<VmCard vm={mockVm} {...mockHandlers} />);
    
    const stopButton = screen.getByText('Stop');
    
    // Click to trigger loading state
    fireEvent.click(stopButton);
    
    // Button should be disabled during loading
    expect(stopButton).toBeDisabled();
  });

  it('formats memory correctly', () => {
    const vmWith8GB = { ...mockVm, memory: 8192 }; // 8GB in MB
    render(<VmCard vm={vmWith8GB} {...mockHandlers} />);
    
    expect(screen.getByText('8GB RAM')).toBeInTheDocument();
  });

  it('handles single vCPU correctly', () => {
    const singleCpuVm = { ...mockVm, vcpus: 1 };
    render(<VmCard vm={singleCpuVm} {...mockHandlers} />);
    
    expect(screen.getByText('1 vCPU')).toBeInTheDocument(); // No 's' for single CPU
  });

  it('handles VM without OS variant', () => {
    const { os_variant, ...vmWithoutVariant } = mockVm;
    render(<VmCard vm={vmWithoutVariant} {...mockHandlers} />);
    
    expect(screen.getByText('Linux')).toBeInTheDocument();
    expect(screen.queryByText('Ubuntu 22.04')).not.toBeInTheDocument();
  });
});
