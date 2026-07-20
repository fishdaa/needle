import { beforeEach, describe, expect, it, vi } from 'vitest'
import { fireEvent, render, screen } from '@testing-library/svelte'
import { invoke } from '@tauri-apps/api/core'
import OptionsWindow from '@/components/OptionsWindow.svelte'
import { defaultKeyboardSettings } from '$lib/keyboardStore.svelte'

vi.mock('@tauri-apps/api/core', () => ({
  invoke: vi.fn()
}))

describe('OptionsWindow', () => {
  beforeEach(() => {
    vi.mocked(invoke).mockImplementation(async (command) => {
      if (command === 'get_keyboard_settings') return defaultKeyboardSettings()
      if (command === 'restore_default_keyboard_settings') return defaultKeyboardSettings()
      if (command === 'save_keyboard_settings') return defaultKeyboardSettings()
      if (command === 'is_autostart_enabled') return false
      return null
    })
  })

  it('renders startup settings and the keyboard options layout', async () => {
    render(OptionsWindow)

    expect(await screen.findByText('Everything Options')).toBeTruthy()
    expect(screen.getByText('Start Toge on system startup')).toBeTruthy()
    expect(screen.getByText('Keyboard')).toBeTruthy()
    await fireEvent.click(screen.getByText('Keyboard'))
    expect(screen.getByDisplayValue('Ctrl+N')).toBeTruthy()
    expect(screen.getByText(/Show commands containing:/)).toBeTruthy()
  })

  it('adds and removes command shortcuts in the draft state', async () => {
    render(OptionsWindow)
    await screen.findByText('Everything Options')
    await fireEvent.click(screen.getByText('Keyboard'))

    await fireEvent.click(screen.getByText('Open Options Window'))
    await fireEvent.click(screen.getByText('Ctrl+Comma (Global)'))
    await fireEvent.click(screen.getByText('Remove'))
    expect(screen.queryByText('Ctrl+Comma (Global)')).toBeNull()

    await fireEvent.click(screen.getByText('Add...'))
    const shortcutInput = screen.getByLabelText('Shortcut key')
    await fireEvent.keyDown(shortcutInput, { key: '.', ctrlKey: true })
    const okButtons = screen.getAllByRole('button', { name: 'OK' })
    await fireEvent.click(okButtons[okButtons.length - 1])

    expect(screen.getByText('Ctrl+Period (Global)')).toBeTruthy()
  })

  it('applies the system startup setting', async () => {
    render(OptionsWindow)
    await screen.findByText('Everything Options')

    await fireEvent.click(screen.getByLabelText('Start Toge on system startup'))
    await fireEvent.click(screen.getByRole('button', { name: 'Apply' }))

    expect(invoke).toHaveBeenCalledWith('set_autostart_enabled', { enabled: true })
  })
})
