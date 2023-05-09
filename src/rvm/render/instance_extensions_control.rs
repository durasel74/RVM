use vulkano::instance::InstanceExtensions;

/// Управляет проверкой допустимости расширений библиотеки Vulkan.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InstanceExtensionsControl {
    supported_extensions: InstanceExtensions,
    minimal_extensions: InstanceExtensions,
    preferred_extensions: InstanceExtensions,
    all_extensions: InstanceExtensions,
    current_extensions: InstanceExtensions,
}

impl InstanceExtensionsControl {
    /// Проверяет допустимые расширения для текущей платформы Windows.
    pub fn new_windows(supported_extensions: &InstanceExtensions) -> InstExtControlCreateResult {
        let minimal_ext_control = if let InstExtControlCreateResult::Ok(iec) = 
            Self::new_windows_minimal(supported_extensions) { iec }
        else { return InstExtControlCreateResult::MinExtNotSupported };

        let preferred_extensions = InstanceExtensions {
            khr_get_physical_device_properties2: true,
            ..InstanceExtensions::empty()
        };
        let checked_extensions = supported_extensions.intersection(&preferred_extensions);
        let all_extensions = minimal_ext_control.minimal_extensions.union(&preferred_extensions);
        let current_extensions = minimal_ext_control.minimal_extensions.union(&checked_extensions);

        let inst_ext_control = InstanceExtensionsControl {
            preferred_extensions,
            all_extensions,
            current_extensions,
            ..minimal_ext_control
        };

        if checked_extensions.contains(&preferred_extensions) {
            InstExtControlCreateResult::Ok(inst_ext_control)
        } else {
            InstExtControlCreateResult::PreferredExtNotSupported(inst_ext_control)
        }
    }

    /// Проверяет минимальный набор расширений для текущей платформы Windows.
    pub fn new_windows_minimal(supported_extensions: &InstanceExtensions) 
    -> InstExtControlCreateResult {
        let minimal_extensions = InstanceExtensions {
            khr_get_surface_capabilities2: true,
            khr_surface: true,
            khr_win32_surface: true,
            ..InstanceExtensions::empty()
        };

        if !supported_extensions.contains(&minimal_extensions) {
            InstExtControlCreateResult::MinExtNotSupported
        } else {
            InstExtControlCreateResult::Ok(InstanceExtensionsControl {
                supported_extensions: supported_extensions.clone(),
                minimal_extensions: minimal_extensions.clone(),
                preferred_extensions: InstanceExtensions::empty(),
                all_extensions: minimal_extensions.clone(),
                current_extensions: minimal_extensions
            })
        }
    }

    // pub fn new_android() {

    // }
}

impl InstanceExtensionsControl {
    /// Возвращает расширения, поддерживаемые библиотекой Vulkan на этом устройстве.
    pub fn supported_extensions(&self) -> &InstanceExtensions {
        &self.supported_extensions
    }

    /// Возвращает минимальный набор расширений необходимый для запуска.
    pub fn minimal_extensions(&self) -> &InstanceExtensions {
        &self.minimal_extensions
    }

    /// Возвращает желаемые расширения, которые могут быть не подключены.
    pub fn preferred_extensions(&self) -> &InstanceExtensions {
        &self.preferred_extensions
    }

    /// Возвращает все возможные расширения, используемые в приложении.
    pub fn all_extensions(&self) -> &InstanceExtensions {
        &self.all_extensions
    }

    /// Возвращает расширения, проверенные на этом устройстве.
    pub fn current_extensions(&self) -> &InstanceExtensions {
        &self.current_extensions
    }

    /// Возвращает необязательные расширения, которые не поддерживаются 
    /// библиотекой Vulkan на этом устройстве.
    pub fn missing_extensions(&self) -> InstanceExtensions {
        self.all_extensions.difference(&self.current_extensions)
    }
}


/// Результат проверки расширений библиотеки.
pub enum InstExtControlCreateResult {
    /// Все расширения можно подключить.
    Ok(InstanceExtensionsControl),

    /// Некоторые необязательные расширения подключить не удалось.
    PreferredExtNotSupported(InstanceExtensionsControl),

    /// Не удалось подключить минимальный набор расширений.
    MinExtNotSupported
}

// mod tests {
    // use vulkano::instance::InstanceExtensions;
    // use crate::rvm::render::{ InstanceExtensionsControl, InstExtControlCreateResult };

    // #[test]
    // fn new_test() {
    //     let minimal_extensions = InstanceExtensions {
    //         khr_get_surface_capabilities2: true,
    //         khr_surface: true,
    //         khr_win32_surface: true,
    //         ..InstanceExtensions::empty()
    //     };

    //     let ok = InstanceExtensionsControl {
    //         supported_extensions: InstanceExtensions::empty(),
    //         minimal_extensions: InstanceExtensions::empty(),
    //         preferred_extensions: InstanceExtensions::empty(),
    //         all_extensions: InstanceExtensions::empty(),
    //         current_extensions: InstanceExtensions::empty()
    //     };
    // }


    // fn create_center<const SIZE: usize>() -> CellMatrix<SIZE> {
    //     let mut matrix = CellMatrix::<SIZE>::new();
    //     matrix.set_cell_value(1, 1, CellValue::Cross);
    //     matrix
    // }

    // fn create_horline<const SIZE: usize>() -> CellMatrix<SIZE> {
    //     let mut matrix = CellMatrix::<SIZE>::new();
    //     matrix.set_cell_value(1, 0, CellValue::Cross);
    //     matrix.set_cell_value(1, 1, CellValue::Cross);
    //     matrix.set_cell_value(1, 2, CellValue::Cross);
    //     matrix
    // }

    // fn create_bottom_horline<const SIZE: usize>() -> CellMatrix<SIZE> {
    //     let mut matrix = CellMatrix::<SIZE>::new();
    //     matrix.set_cell_value(2, 0, CellValue::Cross);
    //     matrix.set_cell_value(2, 1, CellValue::Cross);
    //     matrix.set_cell_value(2, 2, CellValue::Cross);
    //     matrix
    // }

    // fn create_scross<const SIZE: usize>() -> CellMatrix<SIZE> {
    //     let mut matrix = CellMatrix::<SIZE>::new();
    //     matrix.set_cell_value(0, 0, CellValue::Cross);
    //     matrix.set_cell_value(1, 1, CellValue::Cross);
    //     matrix.set_cell_value(2, 2, CellValue::Cross);
    //     matrix.set_cell_value(0, 2, CellValue::Cross);
    //     matrix.set_cell_value(2, 0, CellValue::Cross);
    //     matrix
    // }

    // #[test]
    // fn max_length_horizontal_one_size() {
    //     let mut matrix = CellMatrix::<1>::new();
    //     matrix.set_cell_value(0, 0, CellValue::Zero);
    //     println!("{}", matrix);
    //     let actual = matrix.max_length_horizontal(0, 0);
    //     assert_eq!(1, actual);
    // }

    // #[test]
    // fn max_length_horizontal_three_size_center() {
    //     let matrix = create_center::<3>();
    //     println!("{}", matrix);
    //     let is_left = matrix.max_length_horizontal(1, 0) == 1;
    //     let is_center = matrix.max_length_horizontal(1, 1) == 1;
    //     let is_rigth = matrix.max_length_horizontal(1, 2) == 1;
    //     let is_top = matrix.max_length_horizontal(0, 1) == 3;
    //     let is_bottom = matrix.max_length_horizontal(2, 1) == 3;

    //     let reuslt = is_left && is_center && is_rigth && is_top && is_bottom;
    //     assert!(reuslt);
    // }

    // #[test]
    // fn max_length_horizontal_three_size_horline() {
    //     let matrix = create_horline::<3>();
    //     println!("{}", matrix);
    //     let is_left = matrix.max_length_horizontal(1, 0) == 3;
    //     let is_center = matrix.max_length_horizontal(1, 1) == 3;
    //     let is_rigth = matrix.max_length_horizontal(1, 2) == 3;
    //     let is_top = matrix.max_length_horizontal(0, 1) == 3;
    //     let is_bottom = matrix.max_length_horizontal(2, 1) == 3;

    //     let reuslt = is_left && is_center && is_rigth && is_top && is_bottom;
    //     assert!(reuslt);
    // }
    
    // #[test]
    // fn max_length_horizontal_big_test() {
    //     let mut matrix = CellMatrix::<5>::new();
    //     for i in 0..5 { matrix.set_cell_value(0, i, CellValue::Cross); }

    //     matrix.set_cell_value(1, 0, CellValue::Cross);
    //     matrix.set_cell_value(1, 1, CellValue::Cross);
    //     matrix.set_cell_value(1, 2, CellValue::Zero);
    //     matrix.set_cell_value(1, 3, CellValue::Cross);
    //     matrix.set_cell_value(1, 4, CellValue::Cross);

    //     matrix.set_cell_value(2, 0, CellValue::Cross);
    //     matrix.set_cell_value(2, 1, CellValue::Zero);
    //     matrix.set_cell_value(2, 2, CellValue::Cross);
    //     matrix.set_cell_value(2, 3, CellValue::Zero);
    //     matrix.set_cell_value(2, 4, CellValue::Cross);

    //     matrix.set_cell_value(3, 1, CellValue::Cross);
    //     matrix.set_cell_value(3, 2, CellValue::Cross);
    //     matrix.set_cell_value(3, 3, CellValue::Cross);

    //     matrix.set_cell_value(4, 0, CellValue::None);
    //     matrix.set_cell_value(4, 1, CellValue::Zero);
    //     matrix.set_cell_value(4, 2, CellValue::Zero);
    //     matrix.set_cell_value(4, 3, CellValue::Cross);
    //     matrix.set_cell_value(4, 4, CellValue::Cross);
        
    //     println!("{}", matrix);
    //     let is_all = matrix.max_length_horizontal(0, 3) == 5;
    //     let is_center_zero = matrix.max_length_horizontal(1, 2) == 1;
    //     let is_size_cross = matrix.max_length_horizontal(1, 3) == 2;
    //     let is_one_cross = matrix.max_length_horizontal(2, 4) == 1;
    //     let is_one_zero = matrix.max_length_horizontal(2, 1) == 1;
    //     let is_part_crosses = matrix.max_length_horizontal(3, 1) == 3;
    //     let is_rigth_crosses = matrix.max_length_horizontal(4, 3) == 2;
    //     let is_left_zeros = matrix.max_length_horizontal(4, 2) == 2;

    //     let reuslt = is_all && is_center_zero && is_size_cross && is_one_cross 
    //         && is_one_zero && is_part_crosses && is_rigth_crosses && is_left_zeros;
    //     assert!(reuslt);
    // }

    // #[test]
    // fn one_size_matrix_and_some_functions() {
    //     let mut matrix = CellMatrix::<1>::new();
    //     matrix.clear();
    //     matrix.clear_by_value(CellValue::Cross);
    //     matrix.set_cell_value(0, 0, CellValue::Zero);
    //     matrix.max_length_horizontal(0, 0);
    //     matrix.max_length_vertical(0, 0);
    //     matrix.max_length_diagonal(0, 0);
    //     matrix.max_length_side_diagonal(0, 0);
    //     println!("\n{}\n", matrix);
    // }

    // #[test]
    // #[should_panic(expected = "index out of bounds: the len is 0 but the index is 0")]
    // fn zero_size_matrix_and_some_functions() {
    //     let mut matrix = CellMatrix::<0>::new();
    //     println!("\n{}\n", matrix);
    //     matrix.clear_by_value(CellValue::Cross);
    //     matrix.get_cell_value(0, 0);
    //     matrix.set_cell_value(0, 0, CellValue::Zero);
    //     matrix.max_length_horizontal(0, 0);
    // }
// }