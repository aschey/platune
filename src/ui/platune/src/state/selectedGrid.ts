import { createSlice, PayloadAction } from '@reduxjs/toolkit';

export enum GridType {
  Song,
  Album,
}

interface SelectedGridState {
  selected: GridType;
}

const selectedGridSlice = createSlice({
  name: 'selectedGrid',
  initialState: { selected: GridType.Song },
  reducers: {
    setSelectedGrid: (state, { payload }: PayloadAction<GridType>) => {
      state.selected = payload;
    },
  },
});

export const { setSelectedGrid } = selectedGridSlice.actions;

export const selectGrid = (state: { selectedGrid: SelectedGridState }) => state.selectedGrid.selected;

export default selectedGridSlice.reducer;
