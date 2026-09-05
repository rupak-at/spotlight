import { cleanup, fireEvent, render, screen, waitFor } from '@testing-library/react';
import { App } from './App';
import { api } from './api';

afterEach(cleanup);

it('navigates results with arrows, launches the selected ID, and clears search on Escape', async () => {
  const launch = vi.spyOn(api, 'launch').mockResolvedValue();
  render(<App />);
  const input = screen.getByRole('combobox');
  await waitFor(() => expect(screen.getAllByRole('option').length).toBe(7));
  fireEvent.keyDown(input, { key: 'ArrowDown' });
  expect(input.getAttribute('aria-activedescendant')).toBe('result-1');
  fireEvent.keyDown(input, { key: 'Enter' });
  await waitFor(() => expect(launch).toHaveBeenCalledWith('p2'));
  fireEvent.change(input, { target: { value: 'firefox' } });
  fireEvent.keyDown(input, { key: 'Enter' });
  expect(launch).toHaveBeenCalledTimes(1);
  await waitFor(() => expect(screen.getAllByRole('option').length).toBe(1));
  fireEvent.keyDown(input, { key: 'Escape' });
  expect((input as HTMLInputElement).value).toBe('');
});

it('filters files and opens settings from the keyboard', async () => {
  render(<App />);
  fireEvent.click(screen.getByRole('button', { name: 'Files' }));
  await waitFor(() => expect(screen.getAllByRole('option').length).toBe(1));
  expect(screen.getByText('architecture.md')).toBeTruthy();
  fireEvent.keyDown(screen.getByRole('combobox'), { key: ',', ctrlKey: true });
  expect(screen.getByRole('heading', { name: 'Make it yours' })).toBeTruthy();
  fireEvent.click(screen.getByRole('button', { name: 'Back to search' }));
  expect(screen.getByRole('combobox')).toBeTruthy();
});
