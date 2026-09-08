import { cleanup, fireEvent, render, screen, waitFor } from '@testing-library/react';
import { App } from './App';
import { api } from './api';

afterEach(cleanup);

it('navigates results, launches the selected ID, and hides on Escape with a query', async () => {
  const launch = vi.spyOn(api, 'launch').mockResolvedValue();
  const hide = vi.spyOn(api, 'hide').mockResolvedValue();
  render(<App />);
  const input = screen.getByRole('combobox');
  expect(document.activeElement).toBe(input);
  expect(screen.queryByRole('option')).toBeNull();
  fireEvent.change(input, { target: { value: 'fi' } });
  await waitFor(() => expect(screen.getAllByRole('option').length).toBe(2));
  fireEvent.keyDown(input, { key: 'ArrowDown' });
  expect(input.getAttribute('aria-activedescendant')).toBe('result-1');
  fireEvent.keyDown(input, { key: 'Enter' });
  await waitFor(() => expect(launch).toHaveBeenCalledWith('p2'));
  fireEvent.change(input, { target: { value: 'firefox' } });
  fireEvent.keyDown(input, { key: 'Enter' });
  expect(launch).toHaveBeenCalledTimes(1);
  await waitFor(() => expect(screen.getAllByRole('option').length).toBe(1));
  fireEvent.keyDown(input, { key: 'Escape' });
  expect(hide).toHaveBeenCalledTimes(1);
});

it('filters files and opens settings from the keyboard', async () => {
  const hide = vi.spyOn(api, 'hide').mockResolvedValue();
  render(<App />);
  const input = screen.getByRole('combobox');
  fireEvent.change(input, { target: { value: 'architecture' } });
  await waitFor(() => expect(screen.getAllByRole('option').length).toBe(1));
  fireEvent.keyDown(input, { key: 'Tab', ctrlKey: true });
  fireEvent.keyDown(input, { key: 'Tab', ctrlKey: true });
  await waitFor(() => expect(screen.getAllByRole('option').length).toBe(1));
  expect(screen.getByText('architecture.md')).toBeTruthy();
  fireEvent.keyDown(screen.getByRole('combobox'), { key: ',', ctrlKey: true });
  expect(screen.getByRole('heading', { name: 'Settings' })).toBeTruthy();
  fireEvent.keyDown(screen.getByLabelText('Accent color'), { key: 'Escape' });
  expect(hide).toHaveBeenCalledTimes(1);
  fireEvent.click(screen.getByRole('button', { name: 'Back to search' }));
  await waitFor(() => expect(document.activeElement).toBe(screen.getByRole('combobox')));
});

it('drags a file by ID without opening it, then allows a fresh click', async () => {
  const startDrag = vi.spyOn(api, 'startDrag').mockResolvedValue();
  const launch = vi.spyOn(api, 'launch').mockResolvedValue();
  const callbacks = new Map<string, () => void>();
  vi.spyOn(api, 'on').mockImplementation(async (event, callback) => {
    callbacks.set(event, callback);
    return () => callbacks.delete(event);
  });
  render(<App />);
  fireEvent.change(screen.getByRole('combobox'), { target: { value: 'architecture' } });
  const row = await screen.findByRole('option');
  expect(row.getAttribute('draggable')).toBe('true');
  fireEvent.dragStart(row);
  expect(startDrag).toHaveBeenCalledWith('p7');
  fireEvent.click(row);
  fireEvent.keyDown(screen.getByRole('combobox'), { key: 'Enter' });
  expect(launch).not.toHaveBeenCalled();
  callbacks.get('file-drag-ended')?.();
  fireEvent.click(row);
  expect(launch).not.toHaveBeenCalled();
  fireEvent.mouseDown(row);
  fireEvent.click(row);
  await waitFor(() => expect(launch).toHaveBeenCalledWith('p7'));
});

it('blocks application and stale-result drags and reports native drag failures', async () => {
  const startDrag = vi.spyOn(api, 'startDrag').mockRejectedValue(new Error('File unavailable'));
  render(<App />);
  const input = screen.getByRole('combobox');
  fireEvent.change(input, { target: { value: 'firefox' } });
  const app = await screen.findByRole('option');
  expect(app.getAttribute('draggable')).toBe('false');
  fireEvent.dragStart(app);
  expect(startDrag).not.toHaveBeenCalled();
  fireEvent.change(input, { target: { value: 'architecture' } });
  await screen.findByText('architecture.md');
  fireEvent.dragStart(screen.getByRole('option'));
  expect(await screen.findByRole('alert')).toHaveProperty('textContent', 'Error: File unavailable');
  startDrag.mockClear();
  fireEvent.change(input, { target: { value: 'documents' } });
  const stale = screen.getByRole('option');
  expect(stale.getAttribute('draggable')).toBe('false');
  fireEvent.dragStart(stale);
  expect(startDrag).not.toHaveBeenCalled();
  await screen.findByText('Documents');
  expect(screen.getByRole('option').getAttribute('draggable')).toBe('true');
});
