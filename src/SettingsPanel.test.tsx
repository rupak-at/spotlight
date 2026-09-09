import { cleanup, fireEvent, render, screen, waitFor } from '@testing-library/react';
import { SettingsPanel } from './SettingsPanel';
import { api } from './api';

afterEach(cleanup);

it('saves unlimited scanning while preserving exclusions and shows incomplete coverage', async () => {
  const settings = await api.settings();
  const status = {
    ...(await api.status()),
    truncated: true,
    warnings: ['Index size limit reached'],
  };
  const save = vi.spyOn(api, 'save').mockResolvedValue(settings);
  render(<SettingsPanel settings={settings} status={status} onSave={() => {}} onBack={() => {}} />);
  fireEvent.click(screen.getByRole('button', { name: 'Search & indexing' }));
  expect(screen.getByRole('status').textContent).toContain('Index size limit reached');
  fireEvent.change(screen.getByRole('spinbutton', { name: 'Index limit' }), {
    target: { value: '0' },
  });
  fireEvent.change(screen.getByRole('spinbutton', { name: 'Folder depth' }), {
    target: { value: '0' },
  });
  fireEvent.click(screen.getByRole('button', { name: 'Save changes' }));
  await waitFor(() =>
    expect(save).toHaveBeenCalledWith({ ...settings, max_entries: 0, max_depth: 0 }),
  );
});
