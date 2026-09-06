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
